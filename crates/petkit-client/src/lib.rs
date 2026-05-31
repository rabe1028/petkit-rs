#![forbid(unsafe_code)]

use core::fmt;
#[cfg(any(feature = "async", feature = "blocking"))]
use core::marker::PhantomData;

use md5::{Digest, Md5};
#[cfg(any(feature = "async", feature = "blocking"))]
use nojson::{JsonParseError, RawJsonValue};
#[cfg(any(feature = "async", feature = "blocking"))]
use petkit_protocol::{
    parse_api_response, AuthenticatedProtocol, BaseUrl, DynamicFeeder, DynamicLitter, FeederModel,
    FeederScope, FeederSupportsCalibration, FeederSupportsCallPet, FeederSupportsCamera,
    FeederSupportsFoodReplenished, FeederSupportsSound, FountainScope, LitterModel, LitterScope,
    LitterSupportsCamera, LitterSupportsN50Deodorizer, ManualFeedAmount, PetScope, PublicProtocol,
    PurifierScope, RequestSpec, ResponseParts,
};
use petkit_types::PetkitError;
#[cfg(any(feature = "async", feature = "blocking"))]
use petkit_types::{
    flatten_devices, AccountGroup, CalibrationAction, ClientContext, CloudVideoResponse,
    DeviceCatalog, DeviceFamilyKind, DeviceId, DeviceSummary, FamilyListResponse, FeedDailyList,
    FeedEntryId, FeedIdentifier, FeederCalibrationResponse, FeederCallPetResponse,
    FeederCancelManualFeedResponse, FeederDeviceDetailResponse, FeederDeviceType,
    FeederFoodReplenishedResponse, FeederManualFeedResponse, FeederOpenCameraResponse,
    FeederPlaySoundResponse, FeederRemoveDailyFeedResponse, FeederResetDesiccantResponse,
    FeederRestoreDailyFeedResponse, FeederRestoreFeedResponse, FeederSaveFeedResponse,
    FeederSaveRepeatsResponse, FeederScheduleCompleteResponse, FeederScheduleRemoveResponse,
    FeederScheduleSaveResponse, FeederSetting, FeederStartLiveResponse, FeederSuspendFeedResponse,
    FeederUpdateSettingResponse, FountainDeviceDetailResponse, FountainDeviceType, FountainSetting,
    FountainUpdateSettingResponse, GetDownloadM3u8Response, GetM3u8Response, IotConfigSet,
    IotDeviceInfoV1Response, IotDeviceInfoV2Response, LitterControl, LitterControlDeviceResponse,
    LitterDeviceDetailResponse, LitterDeviceType, LitterOpenCameraResponse,
    LitterResetN50DeodorizerResponse, LitterScheduleCompleteResponse, LitterScheduleRemoveResponse,
    LitterScheduleSaveResponse, LitterSetting, LitterStartLiveResponse,
    LitterUpdateSettingResponse, LoginResponse, PetId, PetUpdateSettingResponse, PetkitDay,
    PurifierControl, PurifierControlDeviceResponse, PurifierDeviceDetailResponse,
    PurifierDeviceType, PurifierSetting, PurifierUpdateSettingResponse, RefreshSessionResponse,
    RegionServersPayload, RegionServersResponse, RepeatSchedule, RequestLoginCodeResponse, Session,
    SoundId,
};

#[cfg(feature = "action-adapter")]
pub mod action_adapter;

#[derive(Debug)]
pub enum ClientError<E> {
    Transport(E),
    Protocol(PetkitError),
}

#[cfg(any(feature = "async", feature = "blocking"))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscoveredDeviceDetail {
    Feeder(FeederDeviceDetailResponse),
    Litter(LitterDeviceDetailResponse),
    Fountain(FountainDeviceDetailResponse),
    Purifier(PurifierDeviceDetailResponse),
}

impl<E> From<PetkitError> for ClientError<E> {
    fn from(value: PetkitError) -> Self {
        Self::Protocol(value)
    }
}

impl<E> fmt::Display for ClientError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(error) => write!(f, "transport error: {error}"),
            Self::Protocol(error) => write!(f, "protocol error: {error}"),
        }
    }
}

impl<E> std::error::Error for ClientError<E> where E: std::error::Error + 'static {}

#[cfg(feature = "async")]
pub trait AsyncTransport {
    type Error;

    fn send(
        &self,
        request: RequestSpec,
    ) -> impl core::future::Future<Output = Result<ResponseParts, Self::Error>>;
}

#[cfg(feature = "blocking")]
pub trait BlockingTransport {
    type Error;

    fn send(&self, request: RequestSpec) -> Result<ResponseParts, Self::Error>;
}

// ---------- Async client ----------

#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncPetkitClient<T> {
    public: PublicProtocol,
    auth: AuthenticatedProtocol,
    transport: T,
}

#[cfg(feature = "async")]
impl<T> AsyncPetkitClient<T> {
    pub fn new(context: ClientContext, regional_base_url: BaseUrl, transport: T) -> Self {
        Self::with_session(context, regional_base_url, "", transport)
    }

    /// Construct a client pre-loaded with a session token (e.g. after restoring
    /// from storage). Use [`set_session`](Self::set_session) to update it later.
    pub fn with_session(
        context: ClientContext,
        regional_base_url: BaseUrl,
        session_id: impl Into<String>,
        transport: T,
    ) -> Self {
        let public = PublicProtocol::new(context.clone());
        let auth = AuthenticatedProtocol::new(context, regional_base_url, session_id);
        Self {
            public,
            auth,
            transport,
        }
    }

    pub fn public(&self) -> &PublicProtocol {
        &self.public
    }

    pub fn authenticated(&self) -> AsyncAuthenticatedClient<'_, T> {
        AsyncAuthenticatedClient { client: self }
    }

    /// Borrow the lower-level protocol builder when you need a raw [`RequestSpec`].
    pub fn authenticated_protocol(&self) -> &AuthenticatedProtocol {
        &self.auth
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Update the session id used by every subsequent authenticated request.
    pub fn set_session(&mut self, session_id: impl Into<String>) {
        self.auth.set_session(session_id);
    }
}

#[cfg(feature = "async")]
impl<T> AsyncPetkitClient<T>
where
    T: AsyncTransport,
{
    pub async fn fetch_region_servers(
        &self,
    ) -> Result<RegionServersPayload, ClientError<T::Error>> {
        let response: RegionServersResponse =
            self.execute_typed(self.public.region_servers()).await?;
        Ok(response.into())
    }

    pub async fn request_login_code(&self, username: &str) -> Result<bool, ClientError<T::Error>> {
        let response: RequestLoginCodeResponse = self
            .execute_typed(self.public.request_login_code(username))
            .await?;
        Ok(response.sent)
    }

    /// Log in with a password. On success the returned [`Session`] is also
    /// persisted on `self`, so subsequent authenticated calls work without
    /// any extra wiring.
    pub async fn login_with_password(
        &mut self,
        username: &str,
        password: &str,
        region: &str,
    ) -> Result<Session, ClientError<T::Error>> {
        let password_md5 = hash_password_md5(password);
        let response: LoginResponse = self
            .execute_typed(
                self.public
                    .login_with_password(username, &password_md5, region),
            )
            .await?;
        let session = response.session;
        self.auth.set_session(&session.id);
        Ok(session)
    }

    /// Log in with a one-time code. The returned [`Session`] is persisted.
    pub async fn login_with_code(
        &mut self,
        username: &str,
        valid_code: &str,
        region: &str,
    ) -> Result<Session, ClientError<T::Error>> {
        let response: LoginResponse = self
            .execute_typed(self.public.login_with_code(username, valid_code, region))
            .await?;
        let session = response.session;
        self.auth.set_session(&session.id);
        Ok(session)
    }

    /// Refresh the current session token; the new token is persisted on `self`.
    pub async fn refresh_session(&mut self) -> Result<Session, ClientError<T::Error>> {
        let response: RefreshSessionResponse =
            self.execute_typed(self.auth.refresh_session()).await?;
        let session = response.session;
        self.auth.set_session(&session.id);
        Ok(session)
    }

    pub async fn family_list(&self) -> Result<Vec<AccountGroup>, ClientError<T::Error>> {
        let response: FamilyListResponse = self.execute_typed(self.auth.family_list()).await?;
        Ok(response.into())
    }

    pub async fn device_list(&self) -> Result<Vec<DeviceSummary>, ClientError<T::Error>> {
        let groups = self.family_list().await?;
        Ok(flatten_devices(&groups))
    }

    pub async fn device_catalog(&self) -> Result<DeviceCatalog, ClientError<T::Error>> {
        let groups = self.family_list().await?;
        Ok(DeviceCatalog::from_groups(&groups))
    }

    pub async fn iot_device_info_v1(&self) -> Result<IotConfigSet, ClientError<T::Error>> {
        let response: IotDeviceInfoV1Response =
            self.execute_typed(self.auth.iot_device_info_v1()).await?;
        Ok(response.into())
    }

    pub async fn iot_device_info_v2(&self) -> Result<IotConfigSet, ClientError<T::Error>> {
        let response: IotDeviceInfoV2Response =
            self.execute_typed(self.auth.iot_device_info_v2()).await?;
        Ok(response.into())
    }

    /// Execute a raw [`RequestSpec`] and parse its JSON envelope into `R`.
    pub async fn execute_typed<R>(&self, request: RequestSpec) -> Result<R, ClientError<T::Error>>
    where
        R: for<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>, Error = JsonParseError>,
    {
        let response = self
            .transport
            .send(request)
            .await
            .map_err(ClientError::Transport)?;
        Ok(parse_api_response(&response)?)
    }
}

// ---------- Blocking client ----------

#[cfg(feature = "blocking")]
#[derive(Debug)]
pub struct BlockingPetkitClient<T> {
    public: PublicProtocol,
    auth: AuthenticatedProtocol,
    transport: T,
}

#[cfg(feature = "blocking")]
impl<T> BlockingPetkitClient<T> {
    pub fn new(context: ClientContext, regional_base_url: BaseUrl, transport: T) -> Self {
        Self::with_session(context, regional_base_url, "", transport)
    }

    pub fn with_session(
        context: ClientContext,
        regional_base_url: BaseUrl,
        session_id: impl Into<String>,
        transport: T,
    ) -> Self {
        let public = PublicProtocol::new(context.clone());
        let auth = AuthenticatedProtocol::new(context, regional_base_url, session_id);
        Self {
            public,
            auth,
            transport,
        }
    }

    pub fn public(&self) -> &PublicProtocol {
        &self.public
    }

    pub fn authenticated(&self) -> BlockingAuthenticatedClient<'_, T> {
        BlockingAuthenticatedClient { client: self }
    }

    /// Borrow the lower-level protocol builder when you need a raw [`RequestSpec`].
    pub fn authenticated_protocol(&self) -> &AuthenticatedProtocol {
        &self.auth
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Update the session id used by every subsequent authenticated request.
    pub fn set_session(&mut self, session_id: impl Into<String>) {
        self.auth.set_session(session_id);
    }
}

#[cfg(feature = "blocking")]
impl<T> BlockingPetkitClient<T>
where
    T: BlockingTransport,
{
    pub fn fetch_region_servers(&self) -> Result<RegionServersPayload, ClientError<T::Error>> {
        let response: RegionServersResponse = self.execute_typed(self.public.region_servers())?;
        Ok(response.into())
    }

    pub fn request_login_code(&self, username: &str) -> Result<bool, ClientError<T::Error>> {
        let response: RequestLoginCodeResponse =
            self.execute_typed(self.public.request_login_code(username))?;
        Ok(response.sent)
    }

    /// Log in with a password. On success the returned [`Session`] is also
    /// persisted on `self`.
    pub fn login_with_password(
        &mut self,
        username: &str,
        password: &str,
        region: &str,
    ) -> Result<Session, ClientError<T::Error>> {
        let password_md5 = hash_password_md5(password);
        let response: LoginResponse = self.execute_typed(self.public.login_with_password(
            username,
            &password_md5,
            region,
        ))?;
        let session = response.session;
        self.auth.set_session(&session.id);
        Ok(session)
    }

    /// Log in with a one-time code. The returned [`Session`] is persisted.
    pub fn login_with_code(
        &mut self,
        username: &str,
        valid_code: &str,
        region: &str,
    ) -> Result<Session, ClientError<T::Error>> {
        let response: LoginResponse =
            self.execute_typed(self.public.login_with_code(username, valid_code, region))?;
        let session = response.session;
        self.auth.set_session(&session.id);
        Ok(session)
    }

    /// Refresh the current session token; the new token is persisted on `self`.
    pub fn refresh_session(&mut self) -> Result<Session, ClientError<T::Error>> {
        let response: RefreshSessionResponse = self.execute_typed(self.auth.refresh_session())?;
        let session = response.session;
        self.auth.set_session(&session.id);
        Ok(session)
    }

    pub fn family_list(&self) -> Result<Vec<AccountGroup>, ClientError<T::Error>> {
        let response: FamilyListResponse = self.execute_typed(self.auth.family_list())?;
        Ok(response.into())
    }

    pub fn device_list(&self) -> Result<Vec<DeviceSummary>, ClientError<T::Error>> {
        let groups = self.family_list()?;
        Ok(flatten_devices(&groups))
    }

    pub fn device_catalog(&self) -> Result<DeviceCatalog, ClientError<T::Error>> {
        let groups = self.family_list()?;
        Ok(DeviceCatalog::from_groups(&groups))
    }

    pub fn iot_device_info_v1(&self) -> Result<IotConfigSet, ClientError<T::Error>> {
        let response: IotDeviceInfoV1Response =
            self.execute_typed(self.auth.iot_device_info_v1())?;
        Ok(response.into())
    }

    pub fn iot_device_info_v2(&self) -> Result<IotConfigSet, ClientError<T::Error>> {
        let response: IotDeviceInfoV2Response =
            self.execute_typed(self.auth.iot_device_info_v2())?;
        Ok(response.into())
    }

    pub fn execute_typed<R>(&self, request: RequestSpec) -> Result<R, ClientError<T::Error>>
    where
        R: for<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>, Error = JsonParseError>,
    {
        let response = self
            .transport
            .send(request)
            .map_err(ClientError::Transport)?;
        Ok(parse_api_response(&response)?)
    }
}

// ---------- Client-backed authenticated scopes ----------

#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncAuthenticatedClient<'a, T> {
    client: &'a AsyncPetkitClient<T>,
}

#[cfg(feature = "async")]
impl<'a, T> AsyncAuthenticatedClient<'a, T> {
    pub fn session_id(&self) -> &str {
        self.client.auth.session_id()
    }

    pub fn protocol(&self) -> &AuthenticatedProtocol {
        &self.client.auth
    }

    pub fn feeder(
        &self,
        device_type: FeederDeviceType,
        device_id: DeviceId,
    ) -> AsyncFeederClient<'a, T> {
        AsyncFeederClient {
            client: self.client,
            device_type,
            device_id,
            _model: PhantomData,
        }
    }

    pub fn feeder_typed<M>(&self, device_id: DeviceId) -> AsyncFeederClient<'a, T, M>
    where
        M: FeederModel,
    {
        AsyncFeederClient {
            client: self.client,
            device_type: M::DEVICE_TYPE,
            device_id,
            _model: PhantomData,
        }
    }

    pub fn litter(
        &self,
        device_type: LitterDeviceType,
        device_id: DeviceId,
    ) -> AsyncLitterClient<'a, T> {
        AsyncLitterClient {
            client: self.client,
            device_type,
            device_id,
            _model: PhantomData,
        }
    }

    pub fn litter_typed<M>(&self, device_id: DeviceId) -> AsyncLitterClient<'a, T, M>
    where
        M: LitterModel,
    {
        AsyncLitterClient {
            client: self.client,
            device_type: M::DEVICE_TYPE,
            device_id,
            _model: PhantomData,
        }
    }

    pub fn fountain(
        &self,
        device_type: FountainDeviceType,
        device_id: DeviceId,
    ) -> AsyncFountainClient<'a, T> {
        AsyncFountainClient {
            client: self.client,
            device_type,
            device_id,
        }
    }

    pub fn purifier(
        &self,
        device_type: PurifierDeviceType,
        device_id: DeviceId,
    ) -> AsyncPurifierClient<'a, T> {
        AsyncPurifierClient {
            client: self.client,
            device_type,
            device_id,
        }
    }

    pub fn pet(&self, pet_id: PetId) -> AsyncPetClient<'a, T> {
        AsyncPetClient {
            client: self.client,
            pet_id,
        }
    }
}

#[cfg(feature = "async")]
impl<T> AsyncAuthenticatedClient<'_, T>
where
    T: AsyncTransport,
{
    pub async fn device_detail_for(
        &self,
        device: &DeviceSummary,
    ) -> Result<DiscoveredDeviceDetail, ClientError<T::Error>> {
        let device_id = device.device_id_value()?;
        match device.device_type.clone().into_family() {
            DeviceFamilyKind::Feeder(device_type) => self
                .feeder(device_type, device_id)
                .device_detail()
                .await
                .map(DiscoveredDeviceDetail::Feeder),
            DeviceFamilyKind::Litter(device_type) => self
                .litter(device_type, device_id)
                .device_detail()
                .await
                .map(DiscoveredDeviceDetail::Litter),
            DeviceFamilyKind::Fountain(device_type) => self
                .fountain(device_type, device_id)
                .device_detail()
                .await
                .map(DiscoveredDeviceDetail::Fountain),
            DeviceFamilyKind::Purifier(device_type) => self
                .purifier(device_type, device_id)
                .device_detail()
                .await
                .map(DiscoveredDeviceDetail::Purifier),
            DeviceFamilyKind::Cozy | DeviceFamilyKind::Pet | DeviceFamilyKind::Unknown(_) => {
                Err(PetkitError::InvalidArgument(format!(
                    "device `{}` does not have a supported device_detail scope",
                    device.device_type.as_str()
                ))
                .into())
            }
        }
    }
}

#[cfg(feature = "blocking")]
#[derive(Debug)]
pub struct BlockingAuthenticatedClient<'a, T> {
    client: &'a BlockingPetkitClient<T>,
}

#[cfg(feature = "blocking")]
impl<'a, T> BlockingAuthenticatedClient<'a, T> {
    pub fn session_id(&self) -> &str {
        self.client.auth.session_id()
    }

    pub fn protocol(&self) -> &AuthenticatedProtocol {
        &self.client.auth
    }

    pub fn feeder(
        &self,
        device_type: FeederDeviceType,
        device_id: DeviceId,
    ) -> BlockingFeederClient<'a, T> {
        BlockingFeederClient {
            client: self.client,
            device_type,
            device_id,
            _model: PhantomData,
        }
    }

    pub fn feeder_typed<M>(&self, device_id: DeviceId) -> BlockingFeederClient<'a, T, M>
    where
        M: FeederModel,
    {
        BlockingFeederClient {
            client: self.client,
            device_type: M::DEVICE_TYPE,
            device_id,
            _model: PhantomData,
        }
    }

    pub fn litter(
        &self,
        device_type: LitterDeviceType,
        device_id: DeviceId,
    ) -> BlockingLitterClient<'a, T> {
        BlockingLitterClient {
            client: self.client,
            device_type,
            device_id,
            _model: PhantomData,
        }
    }

    pub fn litter_typed<M>(&self, device_id: DeviceId) -> BlockingLitterClient<'a, T, M>
    where
        M: LitterModel,
    {
        BlockingLitterClient {
            client: self.client,
            device_type: M::DEVICE_TYPE,
            device_id,
            _model: PhantomData,
        }
    }

    pub fn fountain(
        &self,
        device_type: FountainDeviceType,
        device_id: DeviceId,
    ) -> BlockingFountainClient<'a, T> {
        BlockingFountainClient {
            client: self.client,
            device_type,
            device_id,
        }
    }

    pub fn purifier(
        &self,
        device_type: PurifierDeviceType,
        device_id: DeviceId,
    ) -> BlockingPurifierClient<'a, T> {
        BlockingPurifierClient {
            client: self.client,
            device_type,
            device_id,
        }
    }

    pub fn pet(&self, pet_id: PetId) -> BlockingPetClient<'a, T> {
        BlockingPetClient {
            client: self.client,
            pet_id,
        }
    }
}

#[cfg(feature = "blocking")]
impl<T> BlockingAuthenticatedClient<'_, T>
where
    T: BlockingTransport,
{
    pub fn device_detail_for(
        &self,
        device: &DeviceSummary,
    ) -> Result<DiscoveredDeviceDetail, ClientError<T::Error>> {
        let device_id = device.device_id_value()?;
        match device.device_type.clone().into_family() {
            DeviceFamilyKind::Feeder(device_type) => self
                .feeder(device_type, device_id)
                .device_detail()
                .map(DiscoveredDeviceDetail::Feeder),
            DeviceFamilyKind::Litter(device_type) => self
                .litter(device_type, device_id)
                .device_detail()
                .map(DiscoveredDeviceDetail::Litter),
            DeviceFamilyKind::Fountain(device_type) => self
                .fountain(device_type, device_id)
                .device_detail()
                .map(DiscoveredDeviceDetail::Fountain),
            DeviceFamilyKind::Purifier(device_type) => self
                .purifier(device_type, device_id)
                .device_detail()
                .map(DiscoveredDeviceDetail::Purifier),
            DeviceFamilyKind::Cozy | DeviceFamilyKind::Pet | DeviceFamilyKind::Unknown(_) => {
                Err(PetkitError::InvalidArgument(format!(
                    "device `{}` does not have a supported device_detail scope",
                    device.device_type.as_str()
                ))
                .into())
            }
        }
    }
}

#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncFeederClient<'a, T, M = DynamicFeeder> {
    client: &'a AsyncPetkitClient<T>,
    device_type: FeederDeviceType,
    device_id: DeviceId,
    _model: PhantomData<M>,
}

#[cfg(feature = "async")]
impl<T, M> AsyncFeederClient<'_, T, M> {
    pub fn requests(&self) -> FeederScope<M> {
        self.client
            .auth
            .feeder(self.device_type, self.device_id)
            .with_model()
    }

    pub fn device_detail_request(&self) -> RequestSpec {
        self.requests().device_detail()
    }
}

#[cfg(feature = "async")]
impl<T, M> AsyncFeederClient<'_, T, M>
where
    T: AsyncTransport,
{
    pub async fn device_detail(&self) -> Result<FeederDeviceDetailResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.device_detail_request())
            .await
    }

    pub async fn update_setting(
        &self,
        setting: &FeederSetting,
    ) -> Result<FeederUpdateSettingResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().update_setting(setting))
            .await
    }

    pub async fn cancel_manual_feed(
        &self,
        day: &PetkitDay,
        manual_feed_id: Option<FeedEntryId>,
    ) -> Result<FeederCancelManualFeedResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().cancel_manual_feed(day, manual_feed_id)?)
            .await
    }

    pub async fn remove_daily_feed(
        &self,
        day: &PetkitDay,
        feed_identifier: &FeedIdentifier,
    ) -> Result<FeederRemoveDailyFeedResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().remove_daily_feed(day, feed_identifier))
            .await
    }

    pub async fn restore_daily_feed(
        &self,
        day: &PetkitDay,
        feed_identifier: &FeedIdentifier,
    ) -> Result<FeederRestoreDailyFeedResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().restore_daily_feed(day, feed_identifier))
            .await
    }

    pub async fn save_feed(
        &self,
        feed_daily_list: &FeedDailyList,
    ) -> Result<FeederSaveFeedResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().save_feed(feed_daily_list))
            .await
    }

    pub async fn suspend_feed(&self) -> Result<FeederSuspendFeedResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().suspend_feed())
            .await
    }

    pub async fn restore_feed(&self) -> Result<FeederRestoreFeedResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().restore_feed())
            .await
    }

    pub async fn save_repeats(
        &self,
        schedule: &RepeatSchedule,
    ) -> Result<FeederSaveRepeatsResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().save_repeats(schedule))
            .await
    }

    pub async fn reset_desiccant(
        &self,
    ) -> Result<FeederResetDesiccantResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().reset_desiccant())
            .await
    }

    pub async fn schedule_save(&self) -> Result<FeederScheduleSaveResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().schedule_save())
            .await
    }

    pub async fn schedule_remove(
        &self,
    ) -> Result<FeederScheduleRemoveResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().schedule_remove())
            .await
    }

    pub async fn schedule_complete(
        &self,
    ) -> Result<FeederScheduleCompleteResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().schedule_complete())
            .await
    }
}

#[cfg(feature = "async")]
impl<T, M> AsyncFeederClient<'_, T, M>
where
    T: AsyncTransport,
    M: FeederModel,
{
    pub async fn manual_feed<A>(
        &self,
        amount: A,
        day: &PetkitDay,
    ) -> Result<FeederManualFeedResponse, ClientError<T::Error>>
    where
        A: ManualFeedAmount<M>,
    {
        self.client
            .execute_typed(self.requests().manual_feed(amount, day))
            .await
    }
}

#[cfg(feature = "async")]
impl<T, M> AsyncFeederClient<'_, T, M>
where
    T: AsyncTransport,
    M: FeederSupportsFoodReplenished,
{
    pub async fn food_replenished(
        &self,
    ) -> Result<FeederFoodReplenishedResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().food_replenished())
            .await
    }
}

#[cfg(feature = "async")]
impl<T, M> AsyncFeederClient<'_, T, M>
where
    T: AsyncTransport,
    M: FeederSupportsCalibration,
{
    pub async fn calibration(
        &self,
        action: CalibrationAction,
    ) -> Result<FeederCalibrationResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().calibration(action))
            .await
    }
}

#[cfg(feature = "async")]
impl<T, M> AsyncFeederClient<'_, T, M>
where
    T: AsyncTransport,
    M: FeederSupportsCallPet,
{
    pub async fn call_pet(&self) -> Result<FeederCallPetResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.requests().call_pet()).await
    }
}

#[cfg(feature = "async")]
impl<T, M> AsyncFeederClient<'_, T, M>
where
    T: AsyncTransport,
    M: FeederSupportsSound,
{
    pub async fn play_sound(
        &self,
        sound_id: SoundId,
    ) -> Result<FeederPlaySoundResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().play_sound(sound_id))
            .await
    }
}

#[cfg(feature = "async")]
impl<T, M> AsyncFeederClient<'_, T, M>
where
    M: FeederSupportsCamera,
{
    pub fn open_camera_request(&self) -> RequestSpec {
        self.requests().open_camera()
    }

    pub fn start_live_request(&self) -> RequestSpec {
        self.requests().start_live()
    }

    pub fn cloud_video_request(&self) -> RequestSpec {
        self.requests().cloud_video()
    }

    pub fn get_m3u8_request(&self) -> RequestSpec {
        self.requests().get_m3u8()
    }

    pub fn get_download_m3u8_request(&self) -> RequestSpec {
        self.requests().get_download_m3u8()
    }
}

#[cfg(feature = "async")]
impl<T, M> AsyncFeederClient<'_, T, M>
where
    T: AsyncTransport,
    M: FeederSupportsCamera,
{
    pub async fn open_camera(&self) -> Result<FeederOpenCameraResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.open_camera_request()).await
    }

    pub async fn start_live(&self) -> Result<FeederStartLiveResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.start_live_request()).await
    }

    pub async fn cloud_video(&self) -> Result<CloudVideoResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.cloud_video_request()).await
    }

    pub async fn get_m3u8(&self) -> Result<GetM3u8Response, ClientError<T::Error>> {
        self.client.execute_typed(self.get_m3u8_request()).await
    }

    pub async fn get_download_m3u8(
        &self,
    ) -> Result<GetDownloadM3u8Response, ClientError<T::Error>> {
        self.client
            .execute_typed(self.get_download_m3u8_request())
            .await
    }
}

#[cfg(feature = "blocking")]
#[derive(Debug)]
pub struct BlockingFeederClient<'a, T, M = DynamicFeeder> {
    client: &'a BlockingPetkitClient<T>,
    device_type: FeederDeviceType,
    device_id: DeviceId,
    _model: PhantomData<M>,
}

#[cfg(feature = "blocking")]
impl<T, M> BlockingFeederClient<'_, T, M> {
    pub fn requests(&self) -> FeederScope<M> {
        self.client
            .auth
            .feeder(self.device_type, self.device_id)
            .with_model()
    }

    pub fn device_detail_request(&self) -> RequestSpec {
        self.requests().device_detail()
    }
}

#[cfg(feature = "blocking")]
impl<T, M> BlockingFeederClient<'_, T, M>
where
    T: BlockingTransport,
{
    pub fn device_detail(&self) -> Result<FeederDeviceDetailResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.device_detail_request())
    }

    pub fn update_setting(
        &self,
        setting: &FeederSetting,
    ) -> Result<FeederUpdateSettingResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().update_setting(setting))
    }

    pub fn cancel_manual_feed(
        &self,
        day: &PetkitDay,
        manual_feed_id: Option<FeedEntryId>,
    ) -> Result<FeederCancelManualFeedResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().cancel_manual_feed(day, manual_feed_id)?)
    }

    pub fn remove_daily_feed(
        &self,
        day: &PetkitDay,
        feed_identifier: &FeedIdentifier,
    ) -> Result<FeederRemoveDailyFeedResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().remove_daily_feed(day, feed_identifier))
    }

    pub fn restore_daily_feed(
        &self,
        day: &PetkitDay,
        feed_identifier: &FeedIdentifier,
    ) -> Result<FeederRestoreDailyFeedResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().restore_daily_feed(day, feed_identifier))
    }

    pub fn save_feed(
        &self,
        feed_daily_list: &FeedDailyList,
    ) -> Result<FeederSaveFeedResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().save_feed(feed_daily_list))
    }

    pub fn suspend_feed(&self) -> Result<FeederSuspendFeedResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.requests().suspend_feed())
    }

    pub fn restore_feed(&self) -> Result<FeederRestoreFeedResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.requests().restore_feed())
    }

    pub fn save_repeats(
        &self,
        schedule: &RepeatSchedule,
    ) -> Result<FeederSaveRepeatsResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().save_repeats(schedule))
    }

    pub fn reset_desiccant(&self) -> Result<FeederResetDesiccantResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.requests().reset_desiccant())
    }

    pub fn schedule_save(&self) -> Result<FeederScheduleSaveResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.requests().schedule_save())
    }

    pub fn schedule_remove(&self) -> Result<FeederScheduleRemoveResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.requests().schedule_remove())
    }

    pub fn schedule_complete(
        &self,
    ) -> Result<FeederScheduleCompleteResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().schedule_complete())
    }
}

#[cfg(feature = "blocking")]
impl<T, M> BlockingFeederClient<'_, T, M>
where
    T: BlockingTransport,
    M: FeederModel,
{
    pub fn manual_feed<A>(
        &self,
        amount: A,
        day: &PetkitDay,
    ) -> Result<FeederManualFeedResponse, ClientError<T::Error>>
    where
        A: ManualFeedAmount<M>,
    {
        self.client
            .execute_typed(self.requests().manual_feed(amount, day))
    }
}

#[cfg(feature = "blocking")]
impl<T, M> BlockingFeederClient<'_, T, M>
where
    T: BlockingTransport,
    M: FeederSupportsFoodReplenished,
{
    pub fn food_replenished(&self) -> Result<FeederFoodReplenishedResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().food_replenished())
    }
}

#[cfg(feature = "blocking")]
impl<T, M> BlockingFeederClient<'_, T, M>
where
    T: BlockingTransport,
    M: FeederSupportsCalibration,
{
    pub fn calibration(
        &self,
        action: CalibrationAction,
    ) -> Result<FeederCalibrationResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().calibration(action))
    }
}

#[cfg(feature = "blocking")]
impl<T, M> BlockingFeederClient<'_, T, M>
where
    T: BlockingTransport,
    M: FeederSupportsCallPet,
{
    pub fn call_pet(&self) -> Result<FeederCallPetResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.requests().call_pet())
    }
}

#[cfg(feature = "blocking")]
impl<T, M> BlockingFeederClient<'_, T, M>
where
    T: BlockingTransport,
    M: FeederSupportsSound,
{
    pub fn play_sound(
        &self,
        sound_id: SoundId,
    ) -> Result<FeederPlaySoundResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().play_sound(sound_id))
    }
}

#[cfg(feature = "blocking")]
impl<T, M> BlockingFeederClient<'_, T, M>
where
    M: FeederSupportsCamera,
{
    pub fn open_camera_request(&self) -> RequestSpec {
        self.requests().open_camera()
    }

    pub fn start_live_request(&self) -> RequestSpec {
        self.requests().start_live()
    }

    pub fn cloud_video_request(&self) -> RequestSpec {
        self.requests().cloud_video()
    }

    pub fn get_m3u8_request(&self) -> RequestSpec {
        self.requests().get_m3u8()
    }

    pub fn get_download_m3u8_request(&self) -> RequestSpec {
        self.requests().get_download_m3u8()
    }
}

#[cfg(feature = "blocking")]
impl<T, M> BlockingFeederClient<'_, T, M>
where
    T: BlockingTransport,
    M: FeederSupportsCamera,
{
    pub fn open_camera(&self) -> Result<FeederOpenCameraResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.open_camera_request())
    }

    pub fn start_live(&self) -> Result<FeederStartLiveResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.start_live_request())
    }

    pub fn cloud_video(&self) -> Result<CloudVideoResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.cloud_video_request())
    }

    pub fn get_m3u8(&self) -> Result<GetM3u8Response, ClientError<T::Error>> {
        self.client.execute_typed(self.get_m3u8_request())
    }

    pub fn get_download_m3u8(&self) -> Result<GetDownloadM3u8Response, ClientError<T::Error>> {
        self.client.execute_typed(self.get_download_m3u8_request())
    }
}

#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncLitterClient<'a, T, M = DynamicLitter> {
    client: &'a AsyncPetkitClient<T>,
    device_type: LitterDeviceType,
    device_id: DeviceId,
    _model: PhantomData<M>,
}

#[cfg(feature = "async")]
impl<T, M> AsyncLitterClient<'_, T, M> {
    pub fn requests(&self) -> LitterScope<M> {
        self.client
            .auth
            .litter(self.device_type, self.device_id)
            .with_model()
    }

    pub fn device_detail_request(&self) -> RequestSpec {
        self.requests().device_detail()
    }
}

#[cfg(feature = "async")]
impl<T, M> AsyncLitterClient<'_, T, M>
where
    T: AsyncTransport,
{
    pub async fn device_detail(&self) -> Result<LitterDeviceDetailResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.device_detail_request())
            .await
    }

    pub async fn update_setting(
        &self,
        setting: &LitterSetting,
    ) -> Result<LitterUpdateSettingResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().update_setting(setting))
            .await
    }

    pub async fn control_device(
        &self,
        command: &LitterControl,
    ) -> Result<LitterControlDeviceResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().control_device(command))
            .await
    }

    pub async fn schedule_save(&self) -> Result<LitterScheduleSaveResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().schedule_save())
            .await
    }

    pub async fn schedule_remove(
        &self,
    ) -> Result<LitterScheduleRemoveResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().schedule_remove())
            .await
    }

    pub async fn schedule_complete(
        &self,
    ) -> Result<LitterScheduleCompleteResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().schedule_complete())
            .await
    }
}

#[cfg(feature = "async")]
impl<T, M> AsyncLitterClient<'_, T, M>
where
    T: AsyncTransport,
    M: LitterSupportsN50Deodorizer,
{
    pub async fn reset_n50_deodorizer(
        &self,
    ) -> Result<LitterResetN50DeodorizerResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().reset_n50_deodorizer())
            .await
    }
}

#[cfg(feature = "async")]
impl<T, M> AsyncLitterClient<'_, T, M>
where
    M: LitterSupportsCamera,
{
    pub fn open_camera_request(&self) -> RequestSpec {
        self.requests().open_camera()
    }

    pub fn start_live_request(&self) -> RequestSpec {
        self.requests().start_live()
    }

    pub fn cloud_video_request(&self) -> RequestSpec {
        self.requests().cloud_video()
    }

    pub fn get_m3u8_request(&self) -> RequestSpec {
        self.requests().get_m3u8()
    }

    pub fn get_download_m3u8_request(&self) -> RequestSpec {
        self.requests().get_download_m3u8()
    }
}

#[cfg(feature = "async")]
impl<T, M> AsyncLitterClient<'_, T, M>
where
    T: AsyncTransport,
    M: LitterSupportsCamera,
{
    pub async fn open_camera(&self) -> Result<LitterOpenCameraResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.open_camera_request()).await
    }

    pub async fn start_live(&self) -> Result<LitterStartLiveResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.start_live_request()).await
    }

    pub async fn cloud_video(&self) -> Result<CloudVideoResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.cloud_video_request()).await
    }

    pub async fn get_m3u8(&self) -> Result<GetM3u8Response, ClientError<T::Error>> {
        self.client.execute_typed(self.get_m3u8_request()).await
    }

    pub async fn get_download_m3u8(
        &self,
    ) -> Result<GetDownloadM3u8Response, ClientError<T::Error>> {
        self.client
            .execute_typed(self.get_download_m3u8_request())
            .await
    }
}

#[cfg(feature = "blocking")]
#[derive(Debug)]
pub struct BlockingLitterClient<'a, T, M = DynamicLitter> {
    client: &'a BlockingPetkitClient<T>,
    device_type: LitterDeviceType,
    device_id: DeviceId,
    _model: PhantomData<M>,
}

#[cfg(feature = "blocking")]
impl<T, M> BlockingLitterClient<'_, T, M> {
    pub fn requests(&self) -> LitterScope<M> {
        self.client
            .auth
            .litter(self.device_type, self.device_id)
            .with_model()
    }

    pub fn device_detail_request(&self) -> RequestSpec {
        self.requests().device_detail()
    }
}

#[cfg(feature = "blocking")]
impl<T, M> BlockingLitterClient<'_, T, M>
where
    T: BlockingTransport,
{
    pub fn device_detail(&self) -> Result<LitterDeviceDetailResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.device_detail_request())
    }

    pub fn update_setting(
        &self,
        setting: &LitterSetting,
    ) -> Result<LitterUpdateSettingResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().update_setting(setting))
    }

    pub fn control_device(
        &self,
        command: &LitterControl,
    ) -> Result<LitterControlDeviceResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().control_device(command))
    }

    pub fn schedule_save(&self) -> Result<LitterScheduleSaveResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.requests().schedule_save())
    }

    pub fn schedule_remove(&self) -> Result<LitterScheduleRemoveResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.requests().schedule_remove())
    }

    pub fn schedule_complete(
        &self,
    ) -> Result<LitterScheduleCompleteResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().schedule_complete())
    }
}

#[cfg(feature = "blocking")]
impl<T, M> BlockingLitterClient<'_, T, M>
where
    T: BlockingTransport,
    M: LitterSupportsN50Deodorizer,
{
    pub fn reset_n50_deodorizer(
        &self,
    ) -> Result<LitterResetN50DeodorizerResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().reset_n50_deodorizer())
    }
}

#[cfg(feature = "blocking")]
impl<T, M> BlockingLitterClient<'_, T, M>
where
    M: LitterSupportsCamera,
{
    pub fn open_camera_request(&self) -> RequestSpec {
        self.requests().open_camera()
    }

    pub fn start_live_request(&self) -> RequestSpec {
        self.requests().start_live()
    }

    pub fn cloud_video_request(&self) -> RequestSpec {
        self.requests().cloud_video()
    }

    pub fn get_m3u8_request(&self) -> RequestSpec {
        self.requests().get_m3u8()
    }

    pub fn get_download_m3u8_request(&self) -> RequestSpec {
        self.requests().get_download_m3u8()
    }
}

#[cfg(feature = "blocking")]
impl<T, M> BlockingLitterClient<'_, T, M>
where
    T: BlockingTransport,
    M: LitterSupportsCamera,
{
    pub fn open_camera(&self) -> Result<LitterOpenCameraResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.open_camera_request())
    }

    pub fn start_live(&self) -> Result<LitterStartLiveResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.start_live_request())
    }

    pub fn cloud_video(&self) -> Result<CloudVideoResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.cloud_video_request())
    }

    pub fn get_m3u8(&self) -> Result<GetM3u8Response, ClientError<T::Error>> {
        self.client.execute_typed(self.get_m3u8_request())
    }

    pub fn get_download_m3u8(&self) -> Result<GetDownloadM3u8Response, ClientError<T::Error>> {
        self.client.execute_typed(self.get_download_m3u8_request())
    }
}

#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncFountainClient<'a, T> {
    client: &'a AsyncPetkitClient<T>,
    device_type: FountainDeviceType,
    device_id: DeviceId,
}

#[cfg(feature = "async")]
impl<T> AsyncFountainClient<'_, T> {
    pub fn requests(&self) -> FountainScope {
        self.client.auth.fountain(self.device_type, self.device_id)
    }

    pub fn device_detail_request(&self) -> RequestSpec {
        self.requests().device_detail()
    }
}

#[cfg(feature = "async")]
impl<T> AsyncFountainClient<'_, T>
where
    T: AsyncTransport,
{
    pub async fn device_detail(
        &self,
    ) -> Result<FountainDeviceDetailResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.device_detail_request())
            .await
    }

    pub async fn update_setting(
        &self,
        setting: &FountainSetting,
    ) -> Result<FountainUpdateSettingResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().update_setting(setting))
            .await
    }
}

#[cfg(feature = "blocking")]
#[derive(Debug)]
pub struct BlockingFountainClient<'a, T> {
    client: &'a BlockingPetkitClient<T>,
    device_type: FountainDeviceType,
    device_id: DeviceId,
}

#[cfg(feature = "blocking")]
impl<T> BlockingFountainClient<'_, T> {
    pub fn requests(&self) -> FountainScope {
        self.client.auth.fountain(self.device_type, self.device_id)
    }

    pub fn device_detail_request(&self) -> RequestSpec {
        self.requests().device_detail()
    }
}

#[cfg(feature = "blocking")]
impl<T> BlockingFountainClient<'_, T>
where
    T: BlockingTransport,
{
    pub fn device_detail(&self) -> Result<FountainDeviceDetailResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.device_detail_request())
    }

    pub fn update_setting(
        &self,
        setting: &FountainSetting,
    ) -> Result<FountainUpdateSettingResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().update_setting(setting))
    }
}

#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncPurifierClient<'a, T> {
    client: &'a AsyncPetkitClient<T>,
    device_type: PurifierDeviceType,
    device_id: DeviceId,
}

#[cfg(feature = "async")]
impl<T> AsyncPurifierClient<'_, T> {
    pub fn requests(&self) -> PurifierScope {
        self.client.auth.purifier(self.device_type, self.device_id)
    }

    pub fn device_detail_request(&self) -> RequestSpec {
        self.requests().device_detail()
    }
}

#[cfg(feature = "async")]
impl<T> AsyncPurifierClient<'_, T>
where
    T: AsyncTransport,
{
    pub async fn device_detail(
        &self,
    ) -> Result<PurifierDeviceDetailResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.device_detail_request())
            .await
    }

    pub async fn update_setting(
        &self,
        setting: &PurifierSetting,
    ) -> Result<PurifierUpdateSettingResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().update_setting(setting))
            .await
    }

    pub async fn control_device(
        &self,
        command: &PurifierControl,
    ) -> Result<PurifierControlDeviceResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().control_device(command))
            .await
    }
}

#[cfg(feature = "blocking")]
#[derive(Debug)]
pub struct BlockingPurifierClient<'a, T> {
    client: &'a BlockingPetkitClient<T>,
    device_type: PurifierDeviceType,
    device_id: DeviceId,
}

#[cfg(feature = "blocking")]
impl<T> BlockingPurifierClient<'_, T> {
    pub fn requests(&self) -> PurifierScope {
        self.client.auth.purifier(self.device_type, self.device_id)
    }

    pub fn device_detail_request(&self) -> RequestSpec {
        self.requests().device_detail()
    }
}

#[cfg(feature = "blocking")]
impl<T> BlockingPurifierClient<'_, T>
where
    T: BlockingTransport,
{
    pub fn device_detail(&self) -> Result<PurifierDeviceDetailResponse, ClientError<T::Error>> {
        self.client.execute_typed(self.device_detail_request())
    }

    pub fn update_setting(
        &self,
        setting: &PurifierSetting,
    ) -> Result<PurifierUpdateSettingResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().update_setting(setting))
    }

    pub fn control_device(
        &self,
        command: &PurifierControl,
    ) -> Result<PurifierControlDeviceResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().control_device(command))
    }
}

#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncPetClient<'a, T> {
    client: &'a AsyncPetkitClient<T>,
    pet_id: PetId,
}

#[cfg(feature = "async")]
impl<T> AsyncPetClient<'_, T> {
    pub fn requests(&self) -> PetScope {
        self.client.auth.pet(self.pet_id)
    }
}

#[cfg(feature = "async")]
impl<T> AsyncPetClient<'_, T>
where
    T: AsyncTransport,
{
    pub async fn update_setting(
        &self,
        setting: &petkit_types::PetSetting,
    ) -> Result<PetUpdateSettingResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().update_setting(setting))
            .await
    }
}

#[cfg(feature = "blocking")]
#[derive(Debug)]
pub struct BlockingPetClient<'a, T> {
    client: &'a BlockingPetkitClient<T>,
    pet_id: PetId,
}

#[cfg(feature = "blocking")]
impl<T> BlockingPetClient<'_, T> {
    pub fn requests(&self) -> PetScope {
        self.client.auth.pet(self.pet_id)
    }
}

#[cfg(feature = "blocking")]
impl<T> BlockingPetClient<'_, T>
where
    T: BlockingTransport,
{
    pub fn update_setting(
        &self,
        setting: &petkit_types::PetSetting,
    ) -> Result<PetUpdateSettingResponse, ClientError<T::Error>> {
        self.client
            .execute_typed(self.requests().update_setting(setting))
    }
}

pub fn hash_password_md5(password: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

// ---------- Default client constructors ----------

#[cfg(all(
    feature = "async",
    any(feature = "reqwest-native", feature = "reqwest-async")
))]
pub type ReqwestAsyncPetkitClient = AsyncPetkitClient<reqwest_async::ReqwestAsyncTransport>;

#[cfg(all(
    feature = "async",
    any(feature = "reqwest-native", feature = "reqwest-async")
))]
impl AsyncPetkitClient<reqwest_async::ReqwestAsyncTransport> {
    pub fn new_reqwest(context: ClientContext, regional_base_url: BaseUrl) -> Self {
        Self::new(
            context,
            regional_base_url,
            reqwest_async::ReqwestAsyncTransport::default(),
        )
    }

    pub fn with_reqwest_session(
        context: ClientContext,
        regional_base_url: BaseUrl,
        session_id: impl Into<String>,
    ) -> Self {
        Self::with_session(
            context,
            regional_base_url,
            session_id,
            reqwest_async::ReqwestAsyncTransport::default(),
        )
    }
}

#[cfg(all(
    feature = "blocking",
    any(feature = "reqwest-native", feature = "reqwest-blocking")
))]
pub type ReqwestBlockingPetkitClient =
    BlockingPetkitClient<reqwest_blocking::ReqwestBlockingTransport>;

#[cfg(all(
    feature = "blocking",
    any(feature = "reqwest-native", feature = "reqwest-blocking")
))]
impl BlockingPetkitClient<reqwest_blocking::ReqwestBlockingTransport> {
    pub fn new_reqwest(context: ClientContext, regional_base_url: BaseUrl) -> Self {
        Self::new(
            context,
            regional_base_url,
            reqwest_blocking::ReqwestBlockingTransport::default(),
        )
    }

    pub fn with_reqwest_session(
        context: ClientContext,
        regional_base_url: BaseUrl,
        session_id: impl Into<String>,
    ) -> Self {
        Self::with_session(
            context,
            regional_base_url,
            session_id,
            reqwest_blocking::ReqwestBlockingTransport::default(),
        )
    }
}

#[cfg(all(feature = "blocking", feature = "ureq-blocking"))]
pub type UreqBlockingPetkitClient = BlockingPetkitClient<ureq_blocking::UreqBlockingTransport>;

#[cfg(all(feature = "blocking", feature = "ureq-blocking"))]
impl BlockingPetkitClient<ureq_blocking::UreqBlockingTransport> {
    pub fn new_ureq(context: ClientContext, regional_base_url: BaseUrl) -> Self {
        Self::new(
            context,
            regional_base_url,
            ureq_blocking::UreqBlockingTransport::default(),
        )
    }

    pub fn with_ureq_session(
        context: ClientContext,
        regional_base_url: BaseUrl,
        session_id: impl Into<String>,
    ) -> Self {
        Self::with_session(
            context,
            regional_base_url,
            session_id,
            ureq_blocking::UreqBlockingTransport::default(),
        )
    }
}

#[cfg(feature = "async")]
pub mod host_callback {
    use core::fmt;
    use core::future::Future;

    use petkit_protocol::{RequestSpec, ResponseParts};

    use super::AsyncTransport;

    pub trait HostCallback {
        type Error;

        fn call(
            &self,
            request: RequestSpec,
        ) -> impl Future<Output = Result<ResponseParts, Self::Error>>;
    }

    pub struct HostCallbackTransport<C> {
        callback: C,
    }

    impl<C> HostCallbackTransport<C> {
        pub fn new(callback: C) -> Self {
            Self { callback }
        }

        pub fn callback(&self) -> &C {
            &self.callback
        }
    }

    impl<F> HostCallbackTransport<FnHostCallback<F>> {
        pub fn from_fn(callback: F) -> Self {
            Self::new(FnHostCallback(callback))
        }
    }

    impl<C> fmt::Debug for HostCallbackTransport<C> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("HostCallbackTransport").finish()
        }
    }

    impl<C> AsyncTransport for HostCallbackTransport<C>
    where
        C: HostCallback,
    {
        type Error = C::Error;

        fn send(
            &self,
            request: RequestSpec,
        ) -> impl Future<Output = Result<ResponseParts, Self::Error>> {
            self.callback.call(request)
        }
    }

    pub struct FnHostCallback<F>(F);

    impl<F> fmt::Debug for FnHostCallback<F> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("FnHostCallback").finish()
        }
    }

    impl<F, Fut, E> HostCallback for FnHostCallback<F>
    where
        F: Fn(RequestSpec) -> Fut,
        Fut: Future<Output = Result<ResponseParts, E>>,
    {
        type Error = E;

        fn call(
            &self,
            request: RequestSpec,
        ) -> impl Future<Output = Result<ResponseParts, Self::Error>> {
            (self.0)(request)
        }
    }
}

#[cfg(feature = "blocking")]
pub mod blocking_host_callback {
    use core::fmt;

    use petkit_protocol::{RequestSpec, ResponseParts};

    use super::BlockingTransport;

    pub trait BlockingHostCallback {
        type Error;

        fn call(&self, request: RequestSpec) -> Result<ResponseParts, Self::Error>;
    }

    pub struct BlockingHostCallbackTransport<C> {
        callback: C,
    }

    impl<C> BlockingHostCallbackTransport<C> {
        pub fn new(callback: C) -> Self {
            Self { callback }
        }

        pub fn callback(&self) -> &C {
            &self.callback
        }
    }

    impl<F> BlockingHostCallbackTransport<FnBlockingHostCallback<F>> {
        pub fn from_fn(callback: F) -> Self {
            Self::new(FnBlockingHostCallback(callback))
        }
    }

    impl<C> fmt::Debug for BlockingHostCallbackTransport<C> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("BlockingHostCallbackTransport").finish()
        }
    }

    impl<C> BlockingTransport for BlockingHostCallbackTransport<C>
    where
        C: BlockingHostCallback,
    {
        type Error = C::Error;

        fn send(&self, request: RequestSpec) -> Result<ResponseParts, Self::Error> {
            self.callback.call(request)
        }
    }

    pub struct FnBlockingHostCallback<F>(F);

    impl<F> fmt::Debug for FnBlockingHostCallback<F> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("FnBlockingHostCallback").finish()
        }
    }

    impl<F, E> BlockingHostCallback for FnBlockingHostCallback<F>
    where
        F: Fn(RequestSpec) -> Result<ResponseParts, E>,
    {
        type Error = E;

        fn call(&self, request: RequestSpec) -> Result<ResponseParts, Self::Error> {
            (self.0)(request)
        }
    }
}

// ---------- Reqwest adapters ----------

#[cfg(any(
    all(
        feature = "async",
        any(feature = "reqwest-native", feature = "reqwest-async")
    ),
    all(
        feature = "blocking",
        any(feature = "reqwest-native", feature = "reqwest-blocking")
    )
))]
fn request_method(method: petkit_protocol::HttpMethod) -> reqwest::Method {
    match method {
        petkit_protocol::HttpMethod::Get => reqwest::Method::GET,
        petkit_protocol::HttpMethod::Post => reqwest::Method::POST,
    }
}

#[cfg(any(
    all(
        feature = "async",
        any(feature = "reqwest-native", feature = "reqwest-async")
    ),
    all(
        feature = "blocking",
        any(feature = "reqwest-native", feature = "reqwest-blocking")
    )
))]
fn response_headers(headers: &reqwest::header::HeaderMap) -> Vec<petkit_protocol::Header> {
    headers
        .iter()
        .map(|(name, value)| {
            petkit_protocol::Header::new(
                name.as_str().to_owned(),
                value.to_str().unwrap_or_default(),
            )
        })
        .collect()
}

#[cfg(all(
    feature = "async",
    any(feature = "reqwest-native", feature = "reqwest-async")
))]
pub mod reqwest_async {
    use core::future::Future;

    use petkit_protocol::RequestSpec;

    use super::{request_method, response_headers, AsyncTransport};

    #[derive(Debug)]
    pub struct ReqwestAsyncTransport {
        client: reqwest::Client,
    }

    impl ReqwestAsyncTransport {
        pub fn new(client: reqwest::Client) -> Self {
            Self { client }
        }
    }

    impl Default for ReqwestAsyncTransport {
        fn default() -> Self {
            Self {
                client: reqwest::Client::new(),
            }
        }
    }

    impl AsyncTransport for ReqwestAsyncTransport {
        type Error = reqwest::Error;

        fn send(
            &self,
            request: RequestSpec,
        ) -> impl Future<Output = Result<petkit_protocol::ResponseParts, Self::Error>> {
            let client = self.client.clone();
            let url = request.url().to_owned();
            let RequestSpec {
                method,
                url: _,
                path: _,
                headers,
                query,
                form_fields,
            } = request;

            async move {
                let mut builder = client.request(request_method(method), url);

                if !query.is_empty() {
                    let query_pairs = query
                        .iter()
                        .map(|field| (field.name.as_ref(), field.value.as_str()))
                        .collect::<Vec<_>>();
                    builder = builder.query(&query_pairs);
                }

                if !form_fields.is_empty() {
                    let form_pairs = form_fields
                        .iter()
                        .map(|field| (field.name.as_ref(), field.value.as_str()))
                        .collect::<Vec<_>>();
                    builder = builder.form(&form_pairs);
                }

                for header in headers {
                    builder = builder.header(header.name.as_ref(), header.value);
                }

                let response = builder.send().await?;
                let status = response.status().as_u16();
                let headers = response_headers(response.headers());
                let body = response.bytes().await?.to_vec();
                Ok(petkit_protocol::ResponseParts::new(status, headers, body))
            }
        }
    }
}

#[cfg(all(
    feature = "blocking",
    any(feature = "reqwest-native", feature = "reqwest-blocking")
))]
pub mod reqwest_blocking {
    use petkit_protocol::{RequestSpec, ResponseParts};

    use super::{request_method, response_headers, BlockingTransport};

    #[derive(Debug)]
    pub struct ReqwestBlockingTransport {
        client: reqwest::blocking::Client,
    }

    impl ReqwestBlockingTransport {
        pub fn new(client: reqwest::blocking::Client) -> Self {
            Self { client }
        }
    }

    impl Default for ReqwestBlockingTransport {
        fn default() -> Self {
            Self {
                client: reqwest::blocking::Client::new(),
            }
        }
    }

    impl BlockingTransport for ReqwestBlockingTransport {
        type Error = reqwest::Error;

        fn send(&self, request: RequestSpec) -> Result<ResponseParts, Self::Error> {
            let url = request.url().to_owned();
            let RequestSpec {
                method,
                url: _,
                path: _,
                headers,
                query,
                form_fields,
            } = request;
            let mut builder = self.client.request(request_method(method), url);

            if !query.is_empty() {
                let query_pairs = query
                    .iter()
                    .map(|field| (field.name.as_ref(), field.value.as_str()))
                    .collect::<Vec<_>>();
                builder = builder.query(&query_pairs);
            }

            if !form_fields.is_empty() {
                let form_pairs = form_fields
                    .iter()
                    .map(|field| (field.name.as_ref(), field.value.as_str()))
                    .collect::<Vec<_>>();
                builder = builder.form(&form_pairs);
            }

            for header in headers {
                builder = builder.header(header.name.as_ref(), header.value);
            }

            let response = builder.send()?;
            let status = response.status().as_u16();
            let headers = response_headers(response.headers());
            let body = response.bytes()?.to_vec();
            Ok(ResponseParts::new(status, headers, body))
        }
    }
}

#[cfg(all(feature = "blocking", feature = "ureq-blocking"))]
pub mod ureq_blocking {
    use std::error::Error;
    use std::fmt;
    use std::io::{self, Read};

    use petkit_protocol::{Header, HttpMethod, RequestSpec, ResponseParts};

    use super::BlockingTransport;

    #[derive(Debug)]
    pub enum UreqTransportError {
        Request(Box<ureq::Error>),
        Body(io::Error),
    }

    impl fmt::Display for UreqTransportError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Request(error) => write!(f, "ureq request failed: {error}"),
                Self::Body(error) => write!(f, "failed to read response body: {error}"),
            }
        }
    }

    impl Error for UreqTransportError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                Self::Request(error) => Some(error.as_ref()),
                Self::Body(error) => Some(error),
            }
        }
    }

    impl From<ureq::Error> for UreqTransportError {
        fn from(value: ureq::Error) -> Self {
            Self::Request(Box::new(value))
        }
    }

    impl From<io::Error> for UreqTransportError {
        fn from(value: io::Error) -> Self {
            Self::Body(value)
        }
    }

    pub struct UreqBlockingTransport {
        agent: ureq::Agent,
    }

    impl UreqBlockingTransport {
        pub fn new(agent: ureq::Agent) -> Self {
            Self { agent }
        }
    }

    impl fmt::Debug for UreqBlockingTransport {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("UreqBlockingTransport").finish()
        }
    }

    impl Default for UreqBlockingTransport {
        fn default() -> Self {
            Self {
                agent: ureq::Agent::new(),
            }
        }
    }

    impl BlockingTransport for UreqBlockingTransport {
        type Error = UreqTransportError;

        fn send(&self, request: RequestSpec) -> Result<ResponseParts, Self::Error> {
            let RequestSpec {
                method,
                url,
                path: _,
                headers,
                query,
                form_fields,
            } = request;

            let mut builder = match method {
                HttpMethod::Get => self.agent.get(&url),
                HttpMethod::Post => self.agent.post(&url),
            };

            for header in headers {
                builder = builder.set(header.name.as_ref(), &header.value);
            }
            for field in query {
                builder = builder.query(field.name.as_ref(), &field.value);
            }

            let response = match method {
                HttpMethod::Get => builder.call(),
                HttpMethod::Post => {
                    if form_fields.is_empty() {
                        builder.send_string("")
                    } else {
                        let pairs = form_fields
                            .iter()
                            .map(|field| (field.name.as_ref(), field.value.as_str()))
                            .collect::<Vec<_>>();
                        builder.send_form(&pairs)
                    }
                }
            }
            .map_err(UreqTransportError::from)?;

            let status = response.status();
            let headers = response_headers(&response);

            let mut reader = response.into_reader();
            let mut body = Vec::new();
            reader.read_to_end(&mut body)?;

            Ok(ResponseParts::new(status, headers, body))
        }
    }

    fn response_headers(response: &ureq::Response) -> Vec<Header> {
        let mut headers = Vec::new();
        for name in response.headers_names() {
            for value in response.all(&name) {
                headers.push(Header::new(name.clone(), value.to_owned()));
            }
        }
        headers
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::sync::Mutex;

    #[cfg(any(feature = "async", feature = "blocking"))]
    use std::cell::RefCell;
    #[cfg(feature = "async")]
    use std::future::{ready, Future};
    #[cfg(any(feature = "async", feature = "blocking"))]
    use std::rc::Rc;

    #[cfg(feature = "async")]
    use futures::executor::block_on;

    use petkit_protocol::{
        BaseUrl, D4shFeeder, Header, HttpMethod, RequestSpec, ResponseParts, T6Litter,
    };
    #[cfg(feature = "async")]
    use petkit_types::IotConfigSet;
    use petkit_types::{ClientContext, ClientProfile, DeviceId, DeviceSummary, DeviceType};

    #[cfg(feature = "blocking")]
    use super::blocking_host_callback::BlockingHostCallbackTransport;
    use super::hash_password_md5;
    #[cfg(feature = "async")]
    use super::host_callback::HostCallbackTransport;
    #[cfg(feature = "async")]
    use super::{AsyncPetkitClient, AsyncTransport, DiscoveredDeviceDetail};
    #[cfg(feature = "blocking")]
    use super::{BlockingPetkitClient, BlockingTransport};

    #[cfg(feature = "blocking")]
    #[derive(Debug)]
    struct MockBlockingTransport {
        last_request: Mutex<Option<RequestSpec>>,
        response: ResponseParts,
    }

    #[cfg(feature = "blocking")]
    impl BlockingTransport for MockBlockingTransport {
        type Error = std::convert::Infallible;

        fn send(&self, request: RequestSpec) -> Result<ResponseParts, Self::Error> {
            self.last_request
                .lock()
                .expect("request mutex should not be poisoned")
                .replace(request);
            Ok(self.response.clone())
        }
    }

    #[cfg(feature = "async")]
    #[derive(Debug)]
    struct MockAsyncTransport {
        last_request: Mutex<Option<RequestSpec>>,
        response: ResponseParts,
    }

    #[cfg(feature = "async")]
    impl AsyncTransport for MockAsyncTransport {
        type Error = std::convert::Infallible;

        fn send(
            &self,
            request: RequestSpec,
        ) -> impl Future<Output = Result<ResponseParts, Self::Error>> {
            self.last_request
                .lock()
                .expect("request mutex should not be poisoned")
                .replace(request);
            ready(Ok(self.response.clone()))
        }
    }

    fn ctx() -> ClientContext {
        ClientContext::new(ClientProfile::default(), "Europe/Berlin", "2.0")
    }

    fn regional() -> BaseUrl {
        BaseUrl::Regional("https://api.petkt.com/latest/".into())
    }

    #[test]
    fn md5_hash_matches_python_behavior() {
        assert_eq!(
            hash_password_md5("password"),
            "5f4dcc3b5aa765d61d8327deb882cf99"
        );
    }

    #[cfg(feature = "blocking")]
    #[test]
    fn blocking_client_builds_login_request_and_parses_session() {
        let transport = MockBlockingTransport {
            last_request: Mutex::new(None),
            response: ResponseParts::new(
                200,
                vec![],
                br#"{"result":{"session":{"id":"s1","userId":"u1","expiresIn":3600,"region":"de","createdAt":"2026-05-27T00:00:00.000+0000","refreshedAt":null}}}"#.to_vec(),
            ),
        };
        let mut client = BlockingPetkitClient::new(ctx(), regional(), transport);

        let session = client
            .login_with_password("user@example.com", "password", "DE")
            .expect("login response should parse");
        let request = client
            .transport
            .last_request
            .lock()
            .expect("request mutex should not be poisoned")
            .clone()
            .expect("login request should be captured");

        assert_eq!(session.id, "s1");
        assert_eq!(request.path, "user/login");
        // After login, the session id is persisted on the client.
        assert_eq!(client.authenticated().session_id(), "s1");
    }

    #[cfg(feature = "blocking")]
    #[test]
    fn blocking_client_fetches_region_servers() {
        let transport = MockBlockingTransport {
            last_request: Mutex::new(None),
            response: ResponseParts::new(
                200,
                vec![],
                br#"{"result":{"list":[{"accountType":"overseas","gateway":"https://api.eu-pet.com/latest/","id":"DE","name":"Germany"}]}}"#.to_vec(),
            ),
        };
        let client = BlockingPetkitClient::new(ctx(), regional(), transport);

        let payload = client
            .fetch_region_servers()
            .expect("region server payload should parse");
        assert_eq!(payload.list.len(), 1);
    }

    #[cfg(feature = "blocking")]
    #[test]
    fn blocking_client_request_login_code_propagates_true() {
        let transport = MockBlockingTransport {
            last_request: Mutex::new(None),
            response: ResponseParts::new(200, vec![], br#"{"result":true}"#.to_vec()),
        };
        let client = BlockingPetkitClient::with_session(ctx(), regional(), "session-id", transport);

        let sent = client
            .request_login_code("user@example.com")
            .expect("request login code should parse");
        assert!(sent);
    }

    #[cfg(feature = "blocking")]
    #[test]
    fn blocking_client_request_login_code_propagates_false() {
        let transport = MockBlockingTransport {
            last_request: Mutex::new(None),
            response: ResponseParts::new(200, vec![], br#"{"result":false}"#.to_vec()),
        };
        let client = BlockingPetkitClient::with_session(ctx(), regional(), "session-id", transport);

        let sent = client
            .request_login_code("user@example.com")
            .expect("request login code should parse");
        assert!(!sent);
    }

    #[cfg(feature = "blocking")]
    #[test]
    fn blocking_client_uses_regional_base_for_family_list() {
        let transport = MockBlockingTransport {
            last_request: Mutex::new(None),
            response: ResponseParts::new(200, vec![], br#"{"result":[]}"#.to_vec()),
        };
        let client = BlockingPetkitClient::with_session(ctx(), regional(), "session-id", transport);

        let _ = client.family_list().expect("family list should parse");
        let request = client
            .transport
            .last_request
            .lock()
            .expect("request mutex should not be poisoned")
            .clone()
            .expect("family list request should be captured");

        assert_eq!(
            request.url(),
            "https://api.petkt.com/latest/group/family/list"
        );
    }

    #[cfg(feature = "blocking")]
    #[test]
    fn blocking_client_parses_iot_config() {
        let transport = MockBlockingTransport {
            last_request: Mutex::new(None),
            response: ResponseParts::new(
                200,
                vec![],
                br#"{"result":{"petkit":{"deviceName":"petkit","mqttHost":"mqtt.example"}}}"#
                    .to_vec(),
            ),
        };
        let client = BlockingPetkitClient::with_session(ctx(), regional(), "session-id", transport);

        let config = client
            .iot_device_info_v2()
            .expect("iot config should parse");
        assert_eq!(
            config.petkit,
            Some(petkit_types::IotDeviceInfo {
                created_at: None,
                device_name: Some(String::from("petkit")),
                device_secret: None,
                id: None,
                iot_instance_id: None,
                iot_platform: None,
                mqtt_host: Some(String::from("mqtt.example")),
                mqtt_ip: None,
                product_key: None,
                region_id: None,
                standby_mqtt_host: None,
                standby_mqtt_ip: None,
                device_type_id: None,
            })
        );
    }

    #[cfg(feature = "blocking")]
    #[test]
    fn blocking_host_callback_transport_forwards_request_spec() {
        let seen = Rc::new(RefCell::new(None));
        let transport = BlockingHostCallbackTransport::from_fn({
            let seen = Rc::clone(&seen);
            move |request: RequestSpec| {
                seen.borrow_mut().replace(request);
                Ok::<_, std::convert::Infallible>(ResponseParts::new(
                    200,
                    vec![Header::new("X-Host", "ok")],
                    br#"{"result":true}"#.to_vec(),
                ))
            }
        });
        let request = RequestSpec::new(
            HttpMethod::Post,
            &BaseUrl::Absolute("https://host.example/api".into()),
            "device/action",
        )
        .push_header("X-Test", "forwarded")
        .push_query("q", "1")
        .push_form_field("deviceId", "42")
        .push_form_field("payload", "redacted");

        let response = transport
            .send(request.clone())
            .expect("host call should work");
        let captured = seen
            .borrow()
            .clone()
            .expect("request should be forwarded to callback");

        assert_eq!(response.body, br#"{"result":true}"#);
        assert_eq!(captured.method, request.method);
        assert_eq!(captured.url, request.url);
        assert_eq!(captured.headers, request.headers);
        assert_eq!(captured.query, request.query);
        assert_eq!(captured.form_fields, request.form_fields);
    }

    #[cfg(feature = "blocking")]
    #[test]
    fn blocking_camera_media_helpers_execute_typed_responses() {
        let transport = MockBlockingTransport {
            last_request: Mutex::new(None),
            response: ResponseParts::new(
                200,
                vec![],
                br#"{"result":{"data":{"downloadUrl":"https://media.example/redacted/download.m3u8","aesKey":"aes-redacted"}}}"#.to_vec(),
            ),
        };
        let client = BlockingPetkitClient::with_session(ctx(), regional(), "session-id", transport);

        let response = client
            .authenticated()
            .litter_typed::<T6Litter>(DeviceId::new(42).expect("device id should be valid"))
            .get_download_m3u8()
            .expect("download m3u8 should parse");

        assert_eq!(
            response.primary_url(),
            Some("https://media.example/redacted/download.m3u8")
        );
        let request = client
            .transport
            .last_request
            .lock()
            .expect("request mutex should not be poisoned")
            .clone()
            .expect("m3u8 request should be captured");
        assert_eq!(request.path, "t6/getDownloadM3u8");
    }

    #[cfg(feature = "async")]
    #[test]
    fn async_client_request_login_code_propagates_false() {
        let transport = MockAsyncTransport {
            last_request: Mutex::new(None),
            response: ResponseParts::new(200, vec![], br#"{"result":false}"#.to_vec()),
        };
        let client = AsyncPetkitClient::with_session(ctx(), regional(), "session-id", transport);

        let sent = block_on(client.request_login_code("user@example.com"))
            .expect("request login code should parse");

        assert!(!sent);
    }

    #[cfg(feature = "async")]
    #[test]
    fn async_client_request_login_code_propagates_true() {
        let transport = MockAsyncTransport {
            last_request: Mutex::new(None),
            response: ResponseParts::new(200, vec![], br#"{"result":true}"#.to_vec()),
        };
        let client = AsyncPetkitClient::with_session(ctx(), regional(), "session-id", transport);

        let sent = block_on(client.request_login_code("user@example.com"))
            .expect("request login code should parse");

        assert!(sent);
    }

    #[cfg(feature = "async")]
    #[test]
    fn async_client_uses_regional_base_for_family_list() {
        let transport = MockAsyncTransport {
            last_request: Mutex::new(None),
            response: ResponseParts::new(200, vec![], br#"{"result":[]}"#.to_vec()),
        };
        let client = AsyncPetkitClient::with_session(ctx(), regional(), "session-id", transport);

        let _ = block_on(client.family_list()).expect("family list should parse");
        let request = client
            .transport
            .last_request
            .lock()
            .expect("request mutex should not be poisoned")
            .clone()
            .expect("family list request should be captured");

        assert_eq!(
            request.url(),
            "https://api.petkt.com/latest/group/family/list"
        );
    }

    #[cfg(feature = "async")]
    #[test]
    fn async_client_host_callback_transport_supports_local_state() {
        let seen = Rc::new(RefCell::new(Vec::<String>::new()));
        let transport = HostCallbackTransport::from_fn({
            let seen = Rc::clone(&seen);
            move |request: RequestSpec| {
                seen.borrow_mut().push(request.path.clone());
                ready(Ok::<_, std::convert::Infallible>(ResponseParts::new(
                    200,
                    vec![],
                    br#"{"result":[{"deviceList":[{"deviceId":42,"deviceName":"feeder","deviceType":"d4s","groupId":1,"type":10,"typeCode":20,"uniqueId":"u-42"}],"groupId":1,"name":"home","petList":[]}]}"#.to_vec(),
                )))
            }
        });
        let client = AsyncPetkitClient::with_session(ctx(), regional(), "session-id", transport);

        let devices = block_on(client.device_list()).expect("device list should parse");

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].opaque_id(), "d4s:42");
        assert_eq!(
            seen.borrow().as_slice(),
            &[String::from("group/family/list")]
        );
    }

    #[cfg(feature = "async")]
    #[test]
    fn async_authenticated_client_reads_detail_from_device_summary() {
        let transport = MockAsyncTransport {
            last_request: Mutex::new(None),
            response: ResponseParts::new(
                200,
                vec![],
                br#"{"result":{"id":42,"name":"feeder","settings":{"lightMode":1},"state":{"food":80}}}"#.to_vec(),
            ),
        };
        let client = AsyncPetkitClient::with_session(ctx(), regional(), "session-id", transport);
        let summary = DeviceSummary {
            device_id: 42,
            device_name: Some(String::from("feeder")),
            device_type: DeviceType::D4s,
            group_id: 1,
            device_type_id: Some(10),
            type_code: Some(20),
            unique_id: String::from("u-42"),
        };

        let detail = block_on(client.authenticated().device_detail_for(&summary))
            .expect("device detail should parse");

        assert!(matches!(detail, DiscoveredDeviceDetail::Feeder(_)));
        let request = client
            .transport
            .last_request
            .lock()
            .expect("request mutex should not be poisoned")
            .clone()
            .expect("detail request should be captured");
        assert_eq!(request.path, "d4s/device_detail");
    }

    #[cfg(feature = "async")]
    #[test]
    fn async_client_parses_iot_config() {
        let transport = MockAsyncTransport {
            last_request: Mutex::new(None),
            response: ResponseParts::new(
                200,
                vec![],
                br#"{"result":{"ali":{"deviceName":"ali","mqttHost":"mqtt.example"}}}"#.to_vec(),
            ),
        };
        let client = AsyncPetkitClient::with_session(ctx(), regional(), "session-id", transport);

        let config = block_on(client.iot_device_info_v2()).expect("iot config should parse");
        assert_eq!(
            config,
            IotConfigSet {
                ali: Some(petkit_types::IotDeviceInfo {
                    created_at: None,
                    device_name: Some(String::from("ali")),
                    device_secret: None,
                    id: None,
                    iot_instance_id: None,
                    iot_platform: None,
                    mqtt_host: Some(String::from("mqtt.example")),
                    mqtt_ip: None,
                    product_key: None,
                    region_id: None,
                    standby_mqtt_host: None,
                    standby_mqtt_ip: None,
                    device_type_id: None,
                }),
                petkit: None,
            }
        );
    }

    #[cfg(feature = "async")]
    #[test]
    fn async_camera_media_helpers_execute_typed_responses() {
        let transport = MockAsyncTransport {
            last_request: Mutex::new(None),
            response: ResponseParts::new(
                200,
                vec![],
                br#"{"result":{"data":{"mediaApi":"https://media.example/redacted/cloud.m3u8","aesKey":"aes-redacted"}}}"#.to_vec(),
            ),
        };
        let client = AsyncPetkitClient::with_session(ctx(), regional(), "session-id", transport);

        let response = block_on(
            client
                .authenticated()
                .feeder_typed::<D4shFeeder>(DeviceId::new(42).expect("device id should be valid"))
                .cloud_video(),
        )
        .expect("cloud video should parse");

        assert_eq!(
            response.media_api.as_deref(),
            Some("https://media.example/redacted/cloud.m3u8")
        );
        assert_eq!(response.aes_key.as_deref(), Some("aes-redacted"));
        let request = client
            .transport
            .last_request
            .lock()
            .expect("request mutex should not be poisoned")
            .clone()
            .expect("cloud video request should be captured");
        assert_eq!(request.path, "d4sh/cloud/video");
    }
}

//! Compile-only coverage for request builders across every public and family scope.

#![allow(clippy::print_stdout)]

use std::error::Error;

use petkit_protocol::{
    AuthenticatedProtocol, BaseUrl, D3Feeder, D4sFeeder, D4shFeeder, DualManualFeedAmount,
    FreshElementFeeder, PublicProtocol, RequestSpec, T6Litter,
};
use petkit_types::{
    CalibrationAction, ClientContext, ClientProfile, CustomSetting, CustomSettingValue,
    DeviceFamilyKind, DeviceId, DeviceType, FeedDailyList, FeedEntryId, FeedIdentifier,
    FeederDeviceType, FeederSetting, FountainDeviceType, FountainSetting, LitterControl,
    LitterDeviceType, LitterSetting, PetId, PetSetting, PetWeightGrams, PetkitDay, PurifierControl,
    PurifierDeviceType, PurifierMode, PurifierSetting, RepeatSchedule, ScheduleLimit, SettingInt,
    SettingString, SoundId,
};

fn main() -> Result<(), Box<dyn Error>> {
    let context = ClientContext::new(ClientProfile::default(), "Europe/Berlin", "2.0");
    let public = PublicProtocol::new(context.clone());
    let auth = AuthenticatedProtocol::new(
        context,
        BaseUrl::Regional("https://api.petkt.com/latest/".to_owned()),
        "session-from-example",
    );

    let feeder_d4s = auth.feeder(FeederDeviceType::D4s, DeviceId::new(1_001)?);
    let typed_feeder_d4s = auth.feeder_typed::<D4sFeeder>(DeviceId::new(1_001)?);
    let typed_feeder_d4sh = auth.feeder_typed::<D4shFeeder>(DeviceId::new(1_002)?);
    let typed_feeder_d3 = auth.feeder_typed::<D3Feeder>(DeviceId::new(1_003)?);
    let typed_feeder_legacy = auth.feeder_typed::<FreshElementFeeder>(DeviceId::new(1_004)?);
    let litter_t6 = auth.litter(LitterDeviceType::T6, DeviceId::new(2_001)?);
    let typed_litter_t6 = auth.litter_typed::<T6Litter>(DeviceId::new(2_001)?);
    let fountain_w5 = auth.fountain(FountainDeviceType::W5, DeviceId::new(3_001)?);
    let purifier_k3 = auth.purifier(PurifierDeviceType::K3, DeviceId::new(4_001)?);
    let pet = auth.pet(PetId::new(5_001)?);

    let feed_day = PetkitDay::new("20260529")?;
    let feed_daily_list = FeedDailyList::from_json(r#"[{"amount":10,"time":420}]"#)?;
    let repeat_schedule = RepeatSchedule::new(r#"[{"amount":10,"time":420}]"#)?;
    let fountain_other = FountainSetting::Other(CustomSetting::new(
        "pumpMode",
        CustomSettingValue::String(SettingString::new("smart")?),
    )?);
    let feeder_other = FeederSetting::Other(CustomSetting::new(
        "settings.customMode",
        CustomSettingValue::Int(SettingInt::new(1)?),
    )?);

    let specs = vec![
        public.region_servers(),
        public.request_login_code("user@example.com"),
        public.login_with_password("user@example.com", "md5-password", "DE"),
        public.login_with_code("user@example.com", "000000", "DE"),
        auth.refresh_session(),
        auth.family_list(),
        auth.iot_device_info_v1(),
        auth.iot_device_info_v2(),
        feeder_d4s.device_detail(),
        feeder_d4s.update_setting(&FeederSetting::LightMode(true)),
        feeder_d4s.update_setting(&feeder_other),
        typed_feeder_d4s.manual_feed(DualManualFeedAmount::<D4sFeeder>::new(5, 5)?, &feed_day),
        feeder_d4s.cancel_manual_feed(&feed_day, Some(FeedEntryId::new(42)?))?,
        feeder_d4s.remove_daily_feed(&feed_day, &FeedIdentifier::by_id(42)?),
        feeder_d4s.restore_daily_feed(&feed_day, &FeedIdentifier::by_id(42)?),
        feeder_d4s.save_feed(&feed_daily_list),
        feeder_d4s.suspend_feed(),
        feeder_d4s.restore_feed(),
        feeder_d4s.save_repeats(&repeat_schedule),
        feeder_d4s.reset_desiccant(),
        typed_feeder_d4s.food_replenished(),
        typed_feeder_d3.call_pet(),
        typed_feeder_d3.play_sound(SoundId::new(7)?),
        typed_feeder_d4sh.open_camera(),
        typed_feeder_d4sh.start_live(),
        typed_feeder_d4sh.cloud_video(),
        typed_feeder_d4sh.get_m3u8(),
        typed_feeder_d4sh.get_download_m3u8(),
        typed_feeder_legacy.calibration(CalibrationAction::new(1)?),
        feeder_d4s.schedule_list(ScheduleLimit::new(20)?),
        feeder_d4s.schedule_save(),
        feeder_d4s.schedule_remove(),
        feeder_d4s.schedule_complete(),
        feeder_d4s.schedule_history(),
        litter_t6.device_detail(),
        litter_t6.update_setting(&LitterSetting::AutoWork(true)),
        litter_t6.control_device(&LitterControl::StartCleaning),
        typed_litter_t6.reset_n50_deodorizer(),
        typed_litter_t6.open_camera(),
        typed_litter_t6.start_live(),
        typed_litter_t6.cloud_video(),
        typed_litter_t6.get_m3u8(),
        typed_litter_t6.get_download_m3u8(),
        litter_t6.schedule_list(ScheduleLimit::new(10)?),
        litter_t6.schedule_save(),
        litter_t6.schedule_remove(),
        litter_t6.schedule_complete(),
        litter_t6.schedule_history(),
        fountain_w5.device_detail(),
        fountain_w5.update_setting(&fountain_other),
        purifier_k3.device_detail(),
        purifier_k3.update_setting(&PurifierSetting::Sound(true)),
        purifier_k3.control_device(&PurifierControl::Power(true)),
        pet.update_setting(&PetSetting::Weight(PetWeightGrams::new(4_200)?)),
    ];

    for spec in &specs {
        println!("{}", spec.url());
    }

    let dispatch_device_types = vec![
        DeviceType::D4s,
        DeviceType::T6,
        DeviceType::W5,
        DeviceType::K3,
        DeviceType::Pet,
        DeviceType::Cozy,
        DeviceType::Unknown("mystery-device".to_owned()),
    ];

    for device_type in dispatch_device_types {
        let spec = match device_type.clone().into_family() {
            DeviceFamilyKind::Feeder(kind) => auth
                .feeder(kind, DeviceId::new(6_001)?)
                .schedule_list(ScheduleLimit::new(5)?),
            DeviceFamilyKind::Litter(kind) => auth
                .litter(kind, DeviceId::new(6_002)?)
                .schedule_list(ScheduleLimit::new(5)?),
            DeviceFamilyKind::Fountain(kind) => auth
                .fountain(kind, DeviceId::new(6_003)?)
                .update_setting(&FountainSetting::Other(CustomSetting::new(
                    "filterMode",
                    CustomSettingValue::String(SettingString::new("standard")?),
                )?)),
            DeviceFamilyKind::Purifier(kind) => auth
                .purifier(kind, DeviceId::new(6_004)?)
                .control_device(&PurifierControl::Mode(PurifierMode::Silent)),
            DeviceFamilyKind::Pet => auth
                .pet(PetId::new(6_005)?)
                .update_setting(&PetSetting::Weight(PetWeightGrams::new(3_500)?)),
            DeviceFamilyKind::Cozy | DeviceFamilyKind::Unknown(_) => public.region_servers(),
        };
        println!("{}", spec.url());
    }

    let try_from_specs: [RequestSpec; 4] = [
        auth.feeder(
            FeederDeviceType::try_from(DeviceType::D4s)?,
            DeviceId::new(7_001)?,
        )
        .schedule_list(ScheduleLimit::new(3)?),
        auth.litter(
            LitterDeviceType::try_from(DeviceType::T6)?,
            DeviceId::new(7_002)?,
        )
        .schedule_list(ScheduleLimit::new(3)?),
        auth.fountain(
            FountainDeviceType::try_from(DeviceType::W5)?,
            DeviceId::new(7_003)?,
        )
        .update_setting(&FountainSetting::Other(CustomSetting::new(
            "displayMode",
            CustomSettingValue::String(SettingString::new("eco")?),
        )?)),
        auth.purifier(
            PurifierDeviceType::try_from(DeviceType::K3)?,
            DeviceId::new(7_004)?,
        )
        .control_device(&PurifierControl::Power(true)),
    ];

    for spec in &try_from_specs {
        println!("{}", spec.url());
    }

    Ok(())
}

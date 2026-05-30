use alloc::format;

use core::marker::PhantomData;

use petkit_types::PetkitError;

use super::{DualHopperFeederModel, FeederModel, SingleHopperFeederModel};

pub trait ManualFeedAmount<M: FeederModel>: manual_feed_amount::Sealed<M> {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SingleManualFeedAmount<M> {
    amount: u16,
    _model: PhantomData<M>,
}

impl<M> SingleManualFeedAmount<M>
where
    M: SingleHopperFeederModel,
{
    pub fn new(amount: u16) -> Result<Self, PetkitError> {
        if M::is_valid_single_feed_amount(amount) {
            Ok(Self {
                amount,
                _model: PhantomData,
            })
        } else {
            Err(PetkitError::InvalidArgument(format!(
                "invalid manual feed amount `{amount}` for `{}`",
                M::DEVICE_TYPE.as_str()
            )))
        }
    }

    pub const fn amount(&self) -> u16 {
        self.amount
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DualManualFeedAmount<M> {
    amount1: u16,
    amount2: u16,
    _model: PhantomData<M>,
}

impl<M> DualManualFeedAmount<M>
where
    M: DualHopperFeederModel,
{
    pub fn new(amount1: u16, amount2: u16) -> Result<Self, PetkitError> {
        if amount1 == 0 && amount2 == 0 {
            return Err(PetkitError::InvalidArgument(format!(
                "at least one hopper amount must be non-zero for `{}`",
                M::DEVICE_TYPE.as_str()
            )));
        }

        let valid = |amount: u16| amount <= 10;
        if valid(amount1) && valid(amount2) {
            Ok(Self {
                amount1,
                amount2,
                _model: PhantomData,
            })
        } else {
            Err(PetkitError::InvalidArgument(format!(
                "invalid dual manual feed amount for `{}`",
                M::DEVICE_TYPE.as_str()
            )))
        }
    }

    pub const fn amount1(&self) -> u16 {
        self.amount1
    }

    pub const fn amount2(&self) -> u16 {
        self.amount2
    }
}

pub(super) mod manual_feed_amount {
    use super::{
        DualHopperFeederModel, DualManualFeedAmount, FeederModel, SingleHopperFeederModel,
        SingleManualFeedAmount,
    };

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum Parts {
        Single { amount: u16 },
        Dual { amount1: u16, amount2: u16 },
    }

    pub trait Sealed<M: FeederModel> {
        fn into_parts(self) -> Parts;
    }

    impl<M> Sealed<M> for SingleManualFeedAmount<M>
    where
        M: SingleHopperFeederModel,
    {
        fn into_parts(self) -> Parts {
            Parts::Single {
                amount: self.amount(),
            }
        }
    }

    impl<M> Sealed<M> for DualManualFeedAmount<M>
    where
        M: DualHopperFeederModel,
    {
        fn into_parts(self) -> Parts {
            Parts::Dual {
                amount1: self.amount1(),
                amount2: self.amount2(),
            }
        }
    }
}

impl<M, A> ManualFeedAmount<M> for A
where
    M: FeederModel,
    A: manual_feed_amount::Sealed<M>,
{
}

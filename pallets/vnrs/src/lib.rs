#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        sp_std::vec::Vec,
        traits::{Currency, LockIdentifier, LockableCurrency, WithdrawReasons},
        StorageHasher,
    };
    use frame_system::pallet_prelude::*;

    pub trait NextLockId {
        fn next_lock_id() -> LockIdentifier;
    }

    type VanityName = Vec<u8>;
    type HashedVanityName = [u8; 32];
    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: LockableCurrency<Self::AccountId>;
        type ReservationLifetime: Get<Self::Moment>;
        type RegistrationLifetime: Get<Self::Moment>;
        type ReservationCost: Get<BalanceOf<Self>>;
        type RegistrationOneByteCost: Get<BalanceOf<Self>>;
        type LockIdentifierSource: NextLockId;
    }

    #[pallet::storage]
    #[pallet::getter(fn get_reservation_owner)]
    pub(super) type VanityNameReservationStorage<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        HashedVanityName,
        (T::AccountId, LockIdentifier, T::Moment),
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn get_vanity_name_owner)]
    pub(super) type VanityNameStorage<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        VanityName,
        (T::AccountId, LockIdentifier, T::Moment),
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        VanityNameReserved {
            hash: HashedVanityName,
            owner: T::AccountId,
        },
        VanityNameRegistered {
            name: VanityName,
            owner: T::AccountId,
        },
        ReReserveTry {
            hash: HashedVanityName,
            owner: T::AccountId,
            tried: T::AccountId,
        },
        ReRegisterTry {
            vanity_name: VanityName,
            owner: T::AccountId,
            tried: T::AccountId,
        },
        WrongOwnershipRegisterTry {
            vanity_name: VanityName,
            owner: T::AccountId,
            tried: T::AccountId,
        },
        UnregisterVanityName {
            vanity_name: VanityName,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        AlreadyReserved,
        AlreadyRegistered,
        NameRegistrationError,
        WrongReservationOwnership,
        WrongRegistrationOwnership,
        NoReservation,
        NoRegistrationForRefresh,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Accepts a hash of the name you want to register
        ///
        /// Reservation works only `T::ReservationLifetime::get()` time
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn reservate_vanity_name(
            origin: OriginFor<T>,
            hashed_vanity_name: HashedVanityName,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let now = Self::now();
            match <VanityNameReservationStorage<T>>::try_get(&hashed_vanity_name) {
                Ok((account_id, _lock_id, deadline)) if deadline >= now => {
                    Self::deposit_event(Event::ReReserveTry {
                        hash: hashed_vanity_name,
                        owner: account_id,
                        tried: who,
                    });
                    Err(Error::<T>::AlreadyReserved.into())
                }
                Ok((account_id, lock_id, deadline)) if deadline > now => {
                    T::Currency::remove_lock(lock_id, &account_id);
                    Self::reservate_name(who, hashed_vanity_name);
                    Ok(())
                }
                _ => {
                    Self::reservate_name(who, hashed_vanity_name);
                    Ok(())
                }
            }
        }

        /// Register vanity name based on reservation
        ///
        /// Registration works only `T::RegistrationLifetime::get()` time
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn register_vanity_name(
            origin: OriginFor<T>,
            vanity_name: VanityName,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let hashed_vanity_name = Blake2_256::hash(&vanity_name);
            let now = Self::now();

            match <VanityNameReservationStorage<T>>::try_get(&hashed_vanity_name) {
                Ok((account_id, lock_id, deadline)) if account_id.eq(&who) && deadline >= now => {
                    <VanityNameReservationStorage<T>>::remove(&hashed_vanity_name);
                    Self::try_register_name(who.clone(), vanity_name.clone())?;
                    T::Currency::remove_lock(lock_id, &who);
                    T::Currency::extend_lock(
                        lock_id,
                        &who,
                        BalanceOf::<T>::from(vanity_name.len() as u32),
                        WithdrawReasons::RESERVE,
                    );
                    Ok(())
                }
                Ok((account_id, _lock_id, deadline)) if deadline >= now => {
                    Self::deposit_event(Event::WrongOwnershipRegisterTry {
                        vanity_name,
                        owner: account_id,
                        tried: who,
                    });
                    Err(Error::<T>::WrongReservationOwnership.into())
                }
                _ => Err(Error::<T>::NoReservation.into()),
            }
        }

        /// Refresh registration lifetime
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn refresh_registration_vanity_name(
            origin: OriginFor<T>,
            vanity_name: VanityName,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let now = Self::now();

            match <VanityNameStorage<T>>::try_get(&vanity_name) {
                Ok((account_id, lock_id, deadline)) if account_id.eq(&who) && deadline >= now => {
                    <VanityNameStorage<T>>::mutate(&vanity_name, |value| {
                        value.replace((account_id, lock_id, now + T::RegistrationLifetime::get()))
                    });
                    Ok(())
                }
                Ok((account_id, lock_id, deadline)) if deadline < now => {
                    Self::unregister_name(account_id, lock_id, vanity_name)?;
                    Err(Error::<T>::NoRegistrationForRefresh.into())
                }
                Ok(_) => Err(Error::<T>::WrongRegistrationOwnership.into()),
                Err(_) => Err(Error::<T>::NoRegistrationForRefresh.into()),
            }
        }
    }

    impl<T: Config> Pallet<T> {
        fn now() -> T::Moment {
            <pallet_timestamp::Pallet<T>>::get()
        }

        /// Trying to register name
        /// If name already reserved this function return error
        fn try_register_name(who: T::AccountId, vanity_name: VanityName) -> Result<(), Error<T>> {
            let now = Self::now();

            match <VanityNameStorage<T>>::try_get(&vanity_name) {
                Ok((account_id, _lock_id, deadline)) if deadline >= now => {
                    Self::deposit_event(Event::ReRegisterTry {
                        vanity_name,
                        owner: account_id,
                        tried: who,
                    });
                    Err(Error::<T>::AlreadyRegistered)
                }

                Ok((account_id, lock_id, deadline)) if deadline < now => {
                    T::Currency::remove_lock(lock_id, &account_id);
                    Self::register_name(who, vanity_name);
                    Ok(())
                }
                _ => {
                    Self::register_name(who, vanity_name);
                    Ok(())
                }
            }
        }

        fn register_name(account_id: T::AccountId, vanity_name: VanityName) {
            let lock_id = T::LockIdentifierSource::next_lock_id();
            <VanityNameStorage<T>>::insert(
                vanity_name.clone(),
                (
                    account_id.clone(),
                    lock_id,
                    Self::now() + T::RegistrationLifetime::get(),
                ),
            );
            T::Currency::extend_lock(
                lock_id,
                &account_id,
                BalanceOf::<T>::from(vanity_name.len() as u32) * T::RegistrationOneByteCost::get(),
                WithdrawReasons::RESERVE,
            );
            Self::deposit_event(Event::VanityNameRegistered {
                name: vanity_name,
                owner: account_id,
            });
        }

        /// Reserve name for `account_id`
        /// If name already reserved this function override it
        fn reservate_name(account_id: T::AccountId, hashed_vanity_name: HashedVanityName) {
            let lock_id = T::LockIdentifierSource::next_lock_id();
            <VanityNameReservationStorage<T>>::insert(
                hashed_vanity_name,
                (
                    account_id.clone(),
                    lock_id,
                    Self::now() + T::ReservationLifetime::get(),
                ),
            );
            Self::deposit_event(Event::VanityNameReserved {
                hash: hashed_vanity_name,
                owner: account_id.clone(),
            });
            T::Currency::extend_lock(
                lock_id,
                &account_id,
                T::ReservationCost::get(),
                WithdrawReasons::RESERVE,
            );
        }

        /// Remove registration of name
        fn unregister_name(
            account_id: T::AccountId,
            lock_id: LockIdentifier,
            vanity_name: VanityName,
        ) -> Result<(), Error<T>> {
            <VanityNameStorage<T>>::remove(&vanity_name);
            T::Currency::remove_lock(lock_id, &account_id);
            Self::deposit_event(Event::UnregisterVanityName { vanity_name });
            Ok(())
        }
    }
}

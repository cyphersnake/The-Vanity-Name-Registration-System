use crate::mock::*;
use crate::Error;
use frame_support::{assert_err, assert_ok};
use sp_core::blake2_256;

const VANITY_NAME: &str = "VANITY_NAME";

const USER_1: u64 = 1;
const USER_2: u64 = 2;

#[test]
fn registration_name() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            VnrsModule::get_vanity_name_owner(VANITY_NAME.as_bytes()),
            None
        );
        assert_ok!(VnrsModule::reservate_vanity_name(
            Origin::signed(USER_1),
            blake2_256(VANITY_NAME.as_bytes())
        ));
        assert!(matches!(
            VnrsModule::get_reservation_owner(blake2_256(VANITY_NAME.as_bytes())),
            Some((USER_1, _, _))
        ));
        assert_ok!(VnrsModule::register_vanity_name(
            Origin::signed(USER_1),
            VANITY_NAME.as_bytes().to_vec()
        ));
        assert!(matches!(
            VnrsModule::get_vanity_name_owner(VANITY_NAME.as_bytes()),
            Some((USER_1, _, _))
        ));
    });
}

#[test]
fn registration_without_reservation() {
    new_test_ext().execute_with(|| {
        assert_err!(
            VnrsModule::register_vanity_name(
                Origin::signed(USER_1),
                VANITY_NAME.as_bytes().to_vec()
            ),
            Error::<Test>::NoReservation
        );
    });
}

#[test]
fn registration_with_other_reservation() {
    new_test_ext().execute_with(|| {
        assert_ok!(VnrsModule::reservate_vanity_name(
            Origin::signed(USER_1),
            blake2_256(VANITY_NAME.as_bytes())
        ));
        assert!(matches!(
            VnrsModule::get_reservation_owner(blake2_256(VANITY_NAME.as_bytes())),
            Some((USER_1, _, _))
        ));
        assert_err!(
            VnrsModule::register_vanity_name(
                Origin::signed(USER_2),
                VANITY_NAME.as_bytes().to_vec()
            ),
            Error::<Test>::WrongReservationOwnership
        );
    });
}

#[test]
fn lifetime_of_reservation_test() {
    new_test_ext().execute_with(|| {
        assert_ok!(VnrsModule::reservate_vanity_name(
            Origin::signed(USER_1),
            blake2_256(VANITY_NAME.as_bytes())
        ));
        assert!(matches!(
            VnrsModule::get_reservation_owner(blake2_256(VANITY_NAME.as_bytes())),
            Some((USER_1, _, _))
        ));
        Timestamp::set_timestamp(ReservationLifetime::get() + 1);
        assert_err!(
            VnrsModule::register_vanity_name(
                Origin::signed(USER_1),
                VANITY_NAME.as_bytes().to_vec()
            ),
            Error::<Test>::NoReservation
        );
    });
}

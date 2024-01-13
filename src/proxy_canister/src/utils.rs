use candid::Principal;
use ic_cdk::{api::is_controller, trap};

pub async fn guard_caller_is_controller(caller: &Principal) {
    if !is_controller(caller) {
        trap("Caller is not a controller");
    }
}

pub fn guard_caller_is_not_anonymous(caller: &Principal) {
    if caller.eq(&Principal::anonymous()) {
        trap("Caller is anonymous");
    }
}

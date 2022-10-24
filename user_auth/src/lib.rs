use scrypto::prelude::*;

// This defines our blueprint design that defines the logic of our component. 
blueprint! {
    /// This struct defines the type of vaults and data that our component will hold.
    /// In a permissioned protocol we will want to have some sort of admin badge that will be given to us to
    /// allow us to access permissioned method calls such as approve users who request to be members of this protocol.
    struct UserAuth {
        // This is the ResourceAddress of the admin badge that will allow us to access permissioned method call.
        admin_badge_address: ResourceAddress,
    }

    impl UserAuth {

        // This function will return the ComponentAddress of the component to make it addressable.
        // It will also return us an admin badge through a Bucket.
        pub fn instantiate_user_auth() -> ComponentAddress {

            // The admin badge given to protocol owner.
            let admin_badge: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Admin Badge")
                .metadata("symbol", "AB")
                // Only one will be given at instantiation of the component.
                .initial_supply(1);

            Self {
                admin_badge_address: admin_badge.resource_address(),
            }
            .instantiate()
            .globalize()
        }
    }
}
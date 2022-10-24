use scrypto::prelude::*;

/// This is the NFT struct that will define our temporary badge.
/// The temporary badge will be given to users who request to be a member of this protocol. 
/// The temporary badge is mainly used as a queue to await approval and claim the User Badge to access this
/// protocol when the member is approved.
#[derive(NonFungibleData)]
pub struct TemporaryBadge {
    username: String
}

/// This is the NFT struct that will define our user badge.
/// User badge are approved members of this protocol and members can provide this badge to access
/// authorized features of this protocol.
#[derive(NonFungibleData)]
pub struct UserBadge {
    username: String
}

// This defines our blueprint design that defines the logic of our component. 
blueprint! {
    /// This struct defines the type of vaults and data that our component will hold.
    /// In a permissioned protocol we will want to have some sort of admin badge that will be given to us to
    /// allow us to access permissioned method calls such as approve users who request to be members of this protocol.
    struct UserAuth {
        // This is the ResourceAddress of the admin badge that will allow us to access permissioned method call.
        admin_badge_address: ResourceAddress,
        // This is the ResourceAddress of the temporary badge given to prospective members who request to be members
        // of this protocol. 
        temporary_badge_address: ResourceAddress,
        // This is the badge that will be stored inside a vault of this component. This badge is used to mint or burn
        // TemporaryBadge and UserBadge NFTs.
        component_badge_vault: Vault,
        // This will be a record of pending users requesting to be members of this protocol.
        // This will record the stated username and the associated NFT ID of the TemporaryBadge NFT.
        pending_users: HashMap<String, NonFungibleId>,
        // This will be a record of approved users.
        // This will record the TemporaryBadge NFT ID and the associated UserBadge NFT ID.
        // The reason we set it like this is because when the approved member claims the UserBadge NFT, they will need
        // to deposit the TemporaryBadge NFT so that it can be burnt and retrieve their UserBadge NFT. The component
        // will determin which UserBadge NFT is owed to them based on the the TemporaryBadge NFT they deposit.
        approved_users: HashMap<NonFungibleId, NonFungibleId>,
        // This is the ResourceAddress of the UserBadge NFT that will allow members to access permissioned method call.
        user_badge_address: ResourceAddress,
        // This will be where the UserBadge NFT will be stored where approved members can claim their badges.
        approved_users_vault: Vault,
    }

    impl UserAuth {

        // This function will return the ComponentAddress of the component to make it addressable.
        // It will also return us an admin badge through a Bucket.
        pub fn instantiate_user_auth() -> (ComponentAddress, Bucket) {

            // The admin badge given to protocol owner.
            let admin_badge: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Admin Badge")
                .metadata("symbol", "AB")
                // Only one will be given at instantiation of the component.
                .initial_supply(1);
            
            // The component badge to mint/burn TemporaryBadge/UserBadge NFT.
            // This badge will be stored in one of the component vault.
            let component_badge: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Component Badge")
                .metadata("symbol", "CB")
                // Only one will be sent to one of the component badge.
                .initial_supply(1);
            
            // The temporary badge given to prospective members.
            let temporary_badge: ResourceAddress = ResourceBuilder::new_non_fungible()
                .metadata("name", "Temporary Badge")
                .metadata("symbol", "TB")
                // Mint rule authorized to owner of the component badge.
                .mintable(rule!(require(component_badge.resource_address())), LOCKED)
                // Burn rule authorized to owner of the component badge.
                .burnable(rule!(require(component_badge.resource_address())), LOCKED)
                // No initial supply. Will be minted when "request_user" method is called.
                .no_initial_supply();

            let user_badge: ResourceAddress = ResourceBuilder::new_non_fungible()
                .metadata("name", "User Badge")
                .metadata("symbol", "UB")
                // Mint rule authorized to owner of the component badge.
                .mintable(rule!(require(component_badge.resource_address())), LOCKED)
                // Burn rule authorized to owner of the component badge.
                .burnable(rule!(require(component_badge.resource_address())), LOCKED)
                // No initial supply. Will be minted when "request_user" method is called.
                .no_initial_supply();

            // Access Rule to require admin badge to access the "approve_user" method call.
            let access_rule: AccessRules = AccessRules::new()
                .method("approve_user", rule!(require(admin_badge.resource_address())))
                // All other methods are defaulted to be callable by anyone.
                .default(rule!(allow_all));

            let mut user_auth: UserAuthComponent = Self {
                admin_badge_address: admin_badge.resource_address(),
                temporary_badge_address:temporary_badge,
                component_badge_vault: Vault::with_bucket(component_badge),
                pending_users: HashMap::new(),
                approved_users: HashMap::new(),
                user_badge_address: user_badge,
                approved_users_vault: Vault::new(user_badge),
            }
            .instantiate();
            user_auth.add_access_check(access_rule);
            let user_auth_address: ComponentAddress = user_auth.globalize();

            (user_auth_address, admin_badge)
        }

        /// This method returns a TemporaryBadge NFT in a Bucket.
        pub fn request_user(&mut self, username: String) -> Bucket {
            
            // This will mint us a temporary badge given to users.
            let temporary_badge: Bucket = self.component_badge_vault.authorize(|| {
                let resource_manager: &mut ResourceManager = borrow_resource_manager!(self.temporary_badge_address);
                resource_manager.mint_non_fungible(
                    // The User id
                    &NonFungibleId::random(),
                    // The User data
                    TemporaryBadge {
                        username: username.clone(),
                    },
                )
            });

            // Inserts a record in our `pending_user` data field.
            self.pending_users.insert(username, temporary_badge.non_fungible_id());

            // Returns the TemporaryBadge NFT.
            temporary_badge
        }

        /// This method approves users by minting a UserBadge NFT to be deposited into component's
        /// approved_user_vault. The TemporaryBadge NFT ID will also be recorded in the approved_user data field.
        /// The username will be removed from the pending_users data field.
        pub fn approve_user(&mut self, username: String) {
            
            let temporary_badge_id: &NonFungibleId = self.pending_users.get(&username).unwrap();

            let user_badge: Bucket = self.component_badge_vault.authorize(|| {
                let resource_manager: &mut ResourceManager = borrow_resource_manager!(self.user_badge_address);
                resource_manager.mint_non_fungible(
                    // The User id
                    &NonFungibleId::random(),
                    // The User data
                    UserBadge {
                        username: username.clone(),
                    },
                )
            });

            self.approved_users.insert(temporary_badge_id.clone(), user_badge.non_fungible_id());

            self.approved_users_vault.put(user_badge);

            self.pending_users.remove_entry(&username);

        }

        /// Approved members will call this method to claim ther UserBadge NFT. To do so, they will need to deposit
        /// their TemporaryBadg NFT. The UserBadge NFT will be returned in a Bucket.
        pub fn claim_user(&mut self, temporary_badge: Bucket) -> Bucket {

            // This asserts that the TemporaryBadge NFT deposited was the TemporaryBadge NFT deposited into this
            // component. This prevents a random person depositing an NFT that is not allowed in this protocol.
            assert_eq!(
                temporary_badge.resource_address(), self.temporary_badge_address,
                "Badge does not belong to this protocol!"
            );

            // This retrieves the UserBadge NFT based on the TemporaryBadge NFT ID assocaited with it.
            let user_badge_id: &NonFungibleId = self.approved_users.get(&temporary_badge.non_fungible_id()).unwrap();

            // This takes the UserBadge NFT from the component's approved_user_vault and puts it in a Bucket.
            let user_badge: Bucket = self.approved_users_vault.take_non_fungible(user_badge_id);
            
            self.approved_users.remove_entry(&temporary_badge.non_fungible_id());

            // This authorizes the burn of the TemporaryBadge NFT deposited.
            self.component_badge_vault.authorize(|| temporary_badge.burn());

            // Returns the UserBadge NFT.
            user_badge
        }

        /// This is an example method of what it would look like how members with the UserBadge NFT can access 
        /// permissioned method calls. They will need to provide a Proof of the UserBadge NFT. Unlike the "claim_user"
        /// method call where the user would have to deposit the TemporaryBadge NFT, the Proof is a copy of the 
        /// UserBadge NFT that will drop at the end of the transaction. This is so the user does not have to physically
        /// send the UserBadge NFT itself, only the Proof that they own the UserBadge NFT. 
        pub fn create_auction(&mut self, user_badge: Proof) {
            
            // This validates the Proof that the UserBadge NFT belongs to this protocol, similar to assertion in the 
            // "claim_user" method. 
            user_badge.validate_proof(ProofValidationMode::ValidateResourceAddress(self.user_badge_address))
            .expect("Incorrect User Badge!");

        }
    }
}
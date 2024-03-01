/*
ABOUT THIS CONTRACT...
This contract offers a way for users to sell crytocurrencies privately
among the Geode community. 
- offer your cryptocurrencies for sale
- negotiation, sale and transfer happens offline by what ever method you choose
- no escrow; no currencies or crypto is exchnaged through this contract
*/

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod geode_private_exchange {

    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use ink::storage::StorageVec;
    use ink::env::hash::{Sha2x256, HashOutput};

    // PRELIMINARY DATA STRUCTURES >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct HashVector {
        hashvector: Vec<Hash>
    }
    
    impl Default for HashVector {
        fn default() -> HashVector {
            HashVector {
                hashvector: <Vec<Hash>>::default(),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct Listing {
        listing_id: Hash,
        seller: AccountId,
        offer_coin: Vec<u8>,
        asking_coin: Vec<u8>,
        pair: (Vec<u8>, Vec<u8>),
        price: Balance,
        method: Vec<u8>,
        inventory: Balance,
        country: Vec<u8>,
        city: Vec<u8>,
        notes: Vec<u8>,
        hide: bool, 
    }

    impl Default for Listing {
        fn default() -> Listing {
            let default_addy = "000000000000000000000000000000000000000000000000";
            let default_addy_id32: AccountId = default_addy.as_bytes().try_into().unwrap();
            Listing {
                listing_id: Hash::default(),
                seller: default_addy_id32,
                offer_coin: <Vec<u8>>::default(),
                asking_coin: <Vec<u8>>::default(),
                pair: (<Vec<u8>>::default(), <Vec<u8>>::default()),
                price: Balance::default(),
                method: <Vec<u8>>::default(),
                inventory: Balance::default(),
                country: <Vec<u8>>::default(),
                city: <Vec<u8>>::default(),
                notes: <Vec<u8>>::default(),
                hide: bool::default(),
            }
        }
    }


    // DATA STRUCTURES FOR VIEWING / GET FUNCTIONS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct ViewListings {
        listings: Vec<Listing> 
    }

    impl Default for ViewListings {
        fn default() -> ViewListings {
            ViewListings {
                listings: <Vec<Listing>>::default()
            }
        }
    }


    // EVENT DEFINITIONS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
    
    #[ink(event)]
    // writes a new listing to the blockchain
    pub struct NewListing {
        listing_id: Hash,
        #[ink(topic)]
        seller: AccountId,
        #[ink(topic)]
        pair: (Vec<u8>, Vec<u8>),
        price: Balance,
        inventory: Balance,
        #[ink(topic)]
        country: Vec<u8>,
        city: Vec<u8>
    }

    #[ink(event)]
    // writes a listing update to the blockchain
    pub struct UpdatedListing {
        listing_id: Hash,
        #[ink(topic)]
        seller: AccountId,
        #[ink(topic)]
        pair: (Vec<u8>, Vec<u8>),
        price: Balance,
        inventory: Balance,
        #[ink(topic)]
        country: Vec<u8>,
        city: Vec<u8>
    }


    // ERROR DEFINITIONS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    // Errors that can occur upon calling this contract
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        // a generic error
        GenericError,
        // too much data in the listing
        DataTooLarge,
        // this account has too many listings
        TooManyListings
    }


    // ACTUAL CONTRACT STORAGE >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
    #[ink(storage)]
    pub struct ContractStorage {
        account_listings: Mapping<AccountId, HashVector>,
        listing_details: Mapping<Hash, Listing>,
        all_listings: StorageVec<Hash>,
        all_accounts: StorageVec<AccountId>,
    }


    // CONTRACT LOGIC >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    impl ContractStorage {
        
        // CONSTRUCTORS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // Constructors are implicitly payable when the contract is instantiated.

        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                account_listings: Mapping::default(),
                listing_details: Mapping::default(),
                all_listings: StorageVec::default(),
                all_accounts: StorageVec::default(),
            }
        }


        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // MESSGE FUNCTIONS THAT ALTER CONTRACT STORAGE >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        
        // 0 🟢 New Listing
        #[ink(message)]
        pub fn new_listing (&mut self, 
            offer_coin: Vec<u8>,
            asking_coin: Vec<u8>,
            price: Balance,
            method: Vec<u8>,
            inventory: Balance,
            country: Vec<u8>,
            city: Vec<u8>,
            notes: Vec<u8>, 
        ) -> Result<(), Error> {
            // set up the caller
            let caller = Self::env().caller();
            let rightnow = self.env().block_timestamp();

            // make sure this user has less than 490 listings to avoid overflow
            let my_listings = self.account_listings.get(&caller).unwrap_or_default();
            if my_listings.hashvector.len() > 490 {
                // send an error
                return Err(Error::TooManyListings)
            }
            else {
                // set up clones
                let offer_coin_clone = offer_coin.clone();
                let asking_coin_clone = asking_coin.clone();
                let offer_coin_clone2 = offer_coin.clone();
                let asking_coin_clone2 = asking_coin.clone();
                let offer_coin_clone8 = offer_coin.clone();
                let asking_coin_clone8 = asking_coin.clone();
                let country_clone = country.clone();
                let city_clone = city.clone();

                // make the listing id hash
                let encodable = (caller, offer_coin, asking_coin, rightnow); // Implements `scale::Encode`
                let mut new_id_u8 = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
                ink::env::hash_encoded::<Sha2x256, _>(&encodable, &mut new_id_u8);
                let new_listing_id: Hash = Hash::from(new_id_u8);

                // create the new Listing struct
                let new_listing = Listing {
                    listing_id: new_listing_id,
                    seller: caller,
                    offer_coin: offer_coin_clone,
                    asking_coin: asking_coin_clone,
                    pair: (offer_coin_clone2, asking_coin_clone2),
                    price: price,
                    method: method,
                    inventory: inventory,
                    country: country,
                    city: city,
                    notes: notes,
                    hide: false,
                };

                // UPDATE CONTRACT STORAGE MAPPINGS...

                // account_listings: Mapping<AccountID, HashVector>
                let mut my_listings = self.account_listings.get(&caller).unwrap_or_default();
                my_listings.hashvector.push(new_listing_id);
                if self.account_listings.try_insert(&caller, &my_listings).is_err() {
                    return Err(Error::DataTooLarge);
                }

                // listing_details: Mapping<Hash, Listing>
                if self.listing_details.try_insert(&new_listing_id, &new_listing).is_err() {
                    return Err(Error::DataTooLarge);
                }

                // all_listings StorageVec<Hash>
                if self.all_listings.try_push(&new_listing_id).is_err() {
                    return Err(Error::DataTooLarge);
                }

                // all_accounts StorageVec<AccountId>
                if self.all_accounts.try_push(&caller).is_err() {
                    return Err(Error::DataTooLarge);
                }

                // EMIT EVENT to register the new listing to the chain
                Self::env().emit_event(NewListing {
                    listing_id: new_listing_id,
                    seller: caller,
                    pair: (offer_coin_clone8, asking_coin_clone8),
                    price: price,
                    inventory: inventory,
                    country: country_clone,
                    city: city_clone
                });
                    
                Ok(())
            }
        }


        // 1 🟢 Edit Listing
        #[ink(message)]
        pub fn edit_listing (&mut self, 
            listing_id: Hash,
            price: Balance,
            method: Vec<u8>,
            inventory: Balance,
            country: Vec<u8>,
            city: Vec<u8>,
            notes: Vec<u8>,
            hide: bool,
        ) -> Result<(), Error> {
            // set up the caller
            let caller = Self::env().caller();
            // make sure the caller owns this listing
            // check the account_listings: Mapping<AccountID, HashVector>
            let my_listings = self.account_listings.get(&caller).unwrap_or_default();
            if my_listings.hashvector.contains(&listing_id) {
                let details = self.listing_details.get(&listing_id).unwrap_or_default();
                // make the update structure
                let update = Listing {
                    listing_id: listing_id,
                    seller: caller,
                    offer_coin: details.offer_coin.clone(),
                    asking_coin: details.asking_coin.clone(),
                    pair: details.pair,
                    price: price,
                    method: method,
                    inventory: inventory,
                    country: country.clone(),
                    city: city.clone(),
                    notes: notes,
                    hide: hide,
                };
                // update the mapping listing_details: Mapping<Hash, Listing>
                if self.listing_details.try_insert(&listing_id, &update).is_err() {
                    return Err(Error::DataTooLarge);
                }

                // EMIT EVENT to register the edited listing to the chain
                Self::env().emit_event(UpdatedListing {
                    listing_id: listing_id,
                    seller: caller,
                    pair: (details.offer_coin, details.asking_coin),
                    price: price,
                    inventory: inventory,
                    country: country,
                    city: city
                });

            }
            else {
                // error: Not Your Listing
                return Err(Error::GenericError)
            }

            Ok(())
        }


        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // MESSAGE FUNCTIONS THAT RETRIEVE DATA FROM STORAGE  >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

        // 2 🟢 Browse All Listings
        // include all listings for display
        #[ink(message)]
        pub fn browse_all_listings (&self) -> Vec<Listing> {
            // set up return structure
            let mut results: Vec<Listing> = Vec::new();

            // iterate over all_listings to get the listing details
            if self.all_listings.len() > 0 {
                for i in 0..self.all_listings.len() {
                    // get the listing id
                    let id = self.all_listings.get(i).unwrap();
                    // get the listing details for that id
                    let details = self.listing_details.get(id).unwrap();
                    // check to see it the listing is hidden
                    if details.hide == false {
                        // add the listing details to the results vector
                        results.push(details);
                    }
                }
            }
            // return results
            results
        }


        // 3 🟢 View My Listings
        #[ink(message)]
        pub fn view_my_listings (&self) -> ViewListings {
            // set up the caller
            let caller = Self::env().caller();
            // set up return structures
            let mut my_listings = Vec::new();
            // get the listing IDs for this caller
            let listing_ids = self.account_listings.get(&caller).unwrap_or_default();
            // for each listing ID, get the details and add it to my_listings
            for id in listing_ids.hashvector.iter() {
                let details = self.listing_details.get(id).unwrap_or_default();
                my_listings.push(details);
            }

            // package the results
            let results = ViewListings {
                listings: my_listings 
            };
            // return results
            results
        }

        // 4 🟢 Get all accounts with listings
        #[ink(message)]
        pub fn get_all_accounts (&self) -> Vec<AccountId> {
            // set up return structures
            let mut results = Vec::new();
            // iterate over all_accounts to get the account IDs
            if self.all_accounts.len() > 0 {
                for i in 0..self.all_listings.len() {
                    // get the account id
                    let id = self.all_accounts.get(i).unwrap();
                    results.push(id);
                }
            }
            // return results
            results
        }

        // 5 🟢 Verify that an account has a listing
        #[ink(message)]
        pub fn verify_account(&self, verify_account_id: AccountId) -> u8 {
            // set up return structure
            let mut result: u8 = 0;
            // check the map
            if self.account_listings.contains(&verify_account_id) {
               result = 1;
            }    
            // return results
            result
        }

        // END OF MESSAGE FUNCTIONS

    }
    // END OF CONTRACT LOGIC

}

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
    use ink::env::hash::{Sha2x256, HashOutput};

    // PRELIMINARY DATA STRUCTURES >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[derive(Debug, PartialEq, Eq, Default)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct HashVector {
        hashvector: Vec<Hash>
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
        hide: bool, 
    }

    impl Default for Listing {
        fn default() -> Listing {
            Listing {
                listing_id: Hash::default(),
                seller: AccountId::from([0x0; 32]),
                offer_coin: <Vec<u8>>::default(),
                asking_coin: <Vec<u8>>::default(),
                pair: (<Vec<u8>>::default(), <Vec<u8>>::default()),
                price: Balance::default(),
                method: <Vec<u8>>::default(),
                inventory: Balance::default(),
                country: <Vec<u8>>::default(),
                city: <Vec<u8>>::default(),
                hide: bool::default(),
            }
        }
    }


    // DATA STRUCTURES FOR VIEWING / GET FUNCTIONS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[derive(Clone, Debug, PartialEq, Eq, Default)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std",derive(ink::storage::traits::StorageLayout,))]
    pub struct ViewListings {
        listings: Vec<Listing> 
    }


    // EVENT DEFINITIONS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
    
    #[ink(event)]
    // writes a new listing to the blockchain 
    pub struct NewListing {
        #[ink(topic)]
        listing_id: Hash,
        #[ink(topic)]
        seller: AccountId,
        #[ink(topic)]
        offer_coin: Vec<u8>,
        asking_coin: Vec<u8>,
        pair: (Vec<u8>, Vec<u8>),
        price: Balance,
        method: Vec<u8>,
        inventory: Balance,
        country: Vec<u8>,
        city: Vec<u8>,
        hide: bool, 
    }

    #[ink(event)]
    // writes a listing update to the blockchain
    pub struct UpdatedListing {
        listing_id: Hash,
        #[ink(topic)]
        seller: AccountId,
        #[ink(topic)]
        offer_coin: Vec<u8>,
        asking_coin: Vec<u8>,
        pair: (Vec<u8>, Vec<u8>),
        price: Balance,
        method: Vec<u8>,
        inventory: Balance,
        country: Vec<u8>,
        city: Vec<u8>,
        hide: bool, 
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
        pair_listing_map: Mapping<(Vec<u8>, Vec<u8>), HashVector>,
	    recent_pairs: Vec<(Vec<u8>, Vec<u8>)>,
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
                pair_listing_map: Mapping::default(),
	            recent_pairs: Vec::default(),
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
        ) -> Result<(), Error> {
            // set up the caller
            let caller = Self::env().caller();
            let rightnow = self.env().block_timestamp();

            // if the entries are too big, send an error first
            if offer_coin.len() > 12 || asking_coin.len() > 12 || method.len() > 600
            || country.len() > 40 || city.len() > 60 {
                return Err(Error::DataTooLarge);
            }


            // If this user has 290 listings, kick out the oldest from everywhere
            let mut my_listings = self.account_listings.get(&caller).unwrap_or_default();
            if my_listings.hashvector.len() > 289 {
                // delete their oldest listing from all places in storage 
                let oldest = my_listings.hashvector[0];
                let old_pair = self.listing_details.get(oldest.clone()).unwrap_or_default().pair;
                my_listings.hashvector.remove(0);
                self.account_listings.insert(caller, &my_listings);
                self.listing_details.remove(oldest);
                let mut pairlist = self.pair_listing_map.get(old_pair.clone()).unwrap_or_default();
                // remove from the hashvector by retaining all others 
                pairlist.hashvector.retain(|value| *value != oldest);
                self.pair_listing_map.insert(old_pair, &pairlist);
            }
        
            // make the listing id hash
            // Implements `scale::Encode`
            let encodable = (caller, offer_coin.clone(), asking_coin.clone(), rightnow); 
            let mut new_id_u8 = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
            ink::env::hash_encoded::<Sha2x256, _>(&encodable, &mut new_id_u8);
            let new_listing_id: Hash = Hash::from(new_id_u8);

            // create the new Listing struct
            let new_listing = Listing {
                listing_id: new_listing_id,
                seller: caller,
                offer_coin: offer_coin.clone(),
                asking_coin: asking_coin.clone(),
                pair: (offer_coin.clone(), asking_coin.clone()),
                price: price,
                method: method.clone(),
                inventory: inventory,
                country: country.clone(),
                city: city.clone(),
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

            // add this pair to recent_pairs, kick out the oldest if needed
            let thispair = (offer_coin.clone(), asking_coin.clone());
            if self.recent_pairs.contains(&thispair) {
                // do nothing
            }
            else {
                // if recent_pairs is full, kick out the oldest
                if self.recent_pairs.len() > 57 {
                    let oldest = self.recent_pairs[0].clone();
                    self.recent_pairs.remove(0);
                    // remove the oldest pair from the pair_listing_map
                    self.pair_listing_map.remove(oldest)
                }
                self.recent_pairs.push(thispair.clone());
            }

            // update the pair_listing_map (competitive) 
            // keep the 5 best priced listings for this pair
            // get the listings for this pair
            let mut current_listings = self.pair_listing_map.get(thispair.clone()).unwrap_or_default();
            // if the listings are not full, add this listing
            if current_listings.hashvector.len() < 5 {
                current_listings.hashvector.push(new_listing_id);
                // update the map
                self.pair_listing_map.insert(thispair, &current_listings);
            }
            else {
                // if the listings are full, compete for listing on price
                let highest = current_listings.hashvector[0];
                let mut highest_bid = self.listing_details.get(highest).unwrap_or_default().price;
                let mut highest_index: usize = 0;
                // iterate on the listings to get the highest bidder
                for (i, id) in current_listings.hashvector.iter().enumerate() {
                    let this_bid = self.listing_details.get(id).unwrap_or_default().price;
                    if this_bid > highest_bid {
                        highest_bid = this_bid;
                        highest_index = i;
                    }
                }
                if price > highest_bid {
                    // error - don't let them list a bid that's too high
                    return Err(Error::GenericError);
                }
                else {
                    // kick out the high bidder and add this listing id
                    current_listings.hashvector.remove(highest_index);
                    current_listings.hashvector.push(new_listing_id);
                    // update the map
                    self.pair_listing_map.insert(thispair, &current_listings);
                }
            }
            
            // EMIT EVENT to register the new listing to the chain
            Self::env().emit_event(NewListing {
                listing_id: new_listing_id,
                seller: caller,
                offer_coin: offer_coin.clone(),
                asking_coin: asking_coin.clone(),
                pair: (offer_coin, asking_coin),
                price: price,
                method: method,
                inventory: inventory,
                country: country,
                city: city,
                hide: false, 
            });
                
            Ok(())
            
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
            hide: bool,
        ) -> Result<(), Error> {

            // if the entries are too big, send an error first 
            if method.len() > 600 || country.len() > 40 || city.len() > 60 {
                return Err(Error::DataTooLarge);
            }

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
                    pair: details.pair.clone(),
                    price: price,
                    method: method.clone(),
                    inventory: inventory,
                    country: country.clone(),
                    city: city.clone(),
                    hide: hide,
                };

                // update the mapping listing_details: Mapping<Hash, Listing>
                if self.listing_details.try_insert(&listing_id, &update).is_err() {
                    return Err(Error::DataTooLarge);
                }

                // recompete on price in the pair_listing_map
                // keep the 5 best priced listings for this pair
                // get the listings for this pair
                let thispair = details.pair.clone();
                let mut current_listings = self.pair_listing_map.get(thispair.clone()).unwrap_or_default();
                // if this listing is in the current listings, do nothing
                if current_listings.hashvector.contains(&listing_id) {
                    // do nothing
                }
                else {
                    // let this listing compete as if a new listing
                    // if the listings are not full, add this listing
                    if current_listings.hashvector.len() < 5 {
                        current_listings.hashvector.push(listing_id);
                        // update the map
                        self.pair_listing_map.insert(thispair, &current_listings);
                    }
                    else {
                        // if the listings are full, compete for listing on price
                        let highest = current_listings.hashvector[0];
                        let mut highest_bid = self.listing_details.get(highest).unwrap_or_default().price;
                        let mut highest_index: usize = 0;
                        // iterate on the listings to get the highest bidder
                        for (i, id) in current_listings.hashvector.iter().enumerate() {
                            let this_bid = self.listing_details.get(id).unwrap_or_default().price;
                            if this_bid > highest_bid {
                                highest_bid = this_bid;
                                highest_index = i;
                            }
                        }
                        if price > highest_bid {
                            // error - don't let them list a bid that's too high
                            return Err(Error::GenericError);
                        }
                        else {
                            // kick out the high bidder and add this listing id
                            current_listings.hashvector.remove(highest_index);
                            current_listings.hashvector.push(listing_id);
                            // update the map
                            self.pair_listing_map.insert(thispair, &current_listings);
                        }
                    }
                }


                // EMIT EVENT to register the edited listing to the chain
                Self::env().emit_event(UpdatedListing {
                    listing_id: listing_id,
                    seller: caller,
                    offer_coin: details.offer_coin,
                    asking_coin: details.asking_coin,
                    pair: details.pair,
                    price: price,
                    method: method,
                    inventory: inventory,
                    country: country,
                    city: city,
                    hide: hide, 
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

            // iterate over recent_pairs to get the listing details
            for pair in self.recent_pairs.iter() {
                let listing_ids = self.pair_listing_map.get(pair).unwrap_or_default().hashvector;
                for id in listing_ids.iter() {
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

        // 4 🟢 Verify that an account has a listing
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

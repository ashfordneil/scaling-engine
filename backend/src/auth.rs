//! Module for doing crud operations on the board itself.
use std::collections::HashMap;
use std::path::Path;
use mvdb::Mvdb;
use uuid::Uuid;

use jwt::{encode, Header};

#[derive(Default, Clone, Debug)]
pub struct Auth;

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone, Copy)]
#[serde(tag = "type", content = "id")]
pub enum AuthKey {
    Board(Uuid),
    Tile(Uuid),
}

type JwtString = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    key: AuthKey
}
impl Auth {
    fn storage() -> Mvdb<HashMap<AuthKey, JwtString>> {
        let file = Path::new("target/database/auth.json");
        let STORAGE: Mvdb<HashMap<AuthKey,JwtString>> = Mvdb::from_file(&file)
            .expect("File does not exist, or schema mismatch");
        STORAGE.clone()
    }

    pub fn is_locked(key: AuthKey) -> bool {
        let file = Path::new("target/database/auth.json");
        let my_mvdb: Mvdb<HashMap<AuthKey, JwtString>> = Mvdb::from_file_or_default(&file)
            .expect("Could not write to file");
        let store = my_mvdb.access(|db| db.clone())
        .expect("Failed to access file");
        store.contains_key(&key)
     }

    pub fn lock(key: AuthKey) -> Result<String, ()> {
        if !Auth::is_locked(key) {
            let store = Auth::storage();
            let mut store_from_disk = store.access(|db| db.clone())
                .expect("Failed to access file");

            let claims = key.clone();
            let new_jwt = encode(&Header::default(), &claims, "secret".as_ref());
            if let Ok(new_jwt) = new_jwt {
                store_from_disk.insert(key.clone(), new_jwt.to_string());
                println!("{:?}", new_jwt.to_string());
                Ok(new_jwt.to_string())
            }
            else {
                Err(())
            }


        }
        else {
            Err(())
        }
    }
// Checks if jwt token matches key
    pub fn is_valid(key: AuthKey, jwt: String) -> bool {
        let store = Auth::storage();
         let mut store_from_disk = store.access(|db| db.clone())
             .expect("Failed to access file");
        let stored_jwt = {
            let store = store.access(|db| db.clone())
        .expect("Failed to access file");
            let entry = match store_from_disk.get(&key) {
                Some(val) => Some(val.clone()),
                None => None
            };
            entry
        };
        if let Some(stored_jwt) = stored_jwt {
            if jwt.eq(&stored_jwt) {
                return true;
            }
        }
        return false;
    }

    pub fn unlock(key: AuthKey, jwt: String) -> Result<(), ()> {
        if Auth::is_valid(key, jwt) {
            let store = Auth::storage();
            let mut store = store.access(|db| db.clone())
                .expect("Failed to access file");
            store.remove(&key);
            return Ok(());
        }
        else {
            return Err(());
        }
    }

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_locked() {
        // let uuid = "cbeba719-29dd-4758-9b58-1d9e3b2894d6";
        let uuid = Uuid::new_v4();
        let key = AuthKey::Board(uuid);
        let _jwt = Auth::lock(key);
        assert!(Auth::is_locked(key));
    }

    #[test]
    fn test_unlock() {
        let uuid = Uuid::new_v4();
        let key = AuthKey::Board(uuid);
        let jwt = Auth::lock(key);
        assert!(Auth::is_locked(key));
        let result = Auth::unlock(key, jwt.unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_unlock_jwt() {
        let key1 = AuthKey::Board(Uuid::new_v4());
        assert!(!Auth::is_locked(key1));
        let jwt1 = Auth::lock(key1);

        let key2 = AuthKey::Board(Uuid::new_v4());
        assert!(!Auth::is_locked(key2));
        let jwt2 = Auth::lock(key2);

        assert!(Auth::is_locked(key1));
        assert!(Auth::is_locked(key2));
        assert!(!Auth::unlock(key1, jwt2.unwrap()).is_ok());
        assert!(Auth::unlock(key1, jwt1.unwrap()).is_ok());
    }
}

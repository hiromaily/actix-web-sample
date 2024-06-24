use crate::dbs::conn;
use crate::repositories::{todos, users};
use crate::state;
use crate::toml;
use crate::usecases::{admin, app, auth};
use crate::{hash, jwt};
use cfg_if::cfg_if;
use log::debug;
use sea_orm::{DatabaseConnection, DbErr};
use std::sync::Arc;

async fn new_db_conn(db: &toml::PostgreSQL) -> Result<Option<sea_orm::DatabaseConnection>, DbErr> {
    if db.enabled {
        let connected = conn::get_conn(&db.user, &db.password, &db.host, &db.dbname).await?;
        return Ok(Some(connected));
    }
    Ok(None)
}

// error would occur if TodoRepository has clone trait as supertrait
// fn new_todos_repository(&self) -> Box<dyn todo_repository::TodoRepository> {
//     if self.conf.db.enabled {
//         return Box::new(todo_repository::TodoRepositoryForDB::new());
//     } else {
//         return Box::new(todo_repository::TodoRepositoryForMemory::new());
//     }
// }
async fn new_todos_repository(
    db_conn: Option<DatabaseConnection>,
) -> Result<Arc<dyn todos::TodoRepository>, DbErr> {
    if let Some(conn) = db_conn {
        return Ok(Arc::new(todos::TodoRepositoryForDB::new(conn)));
    }
    Ok(Arc::new(todos::TodoRepositoryForMemory::new()))
}

async fn new_users_repository(
    db_conn: Option<sea_orm::DatabaseConnection>,
) -> Result<Arc<dyn users::UserRepository>, DbErr> {
    if let Some(conn) = db_conn {
        return Ok(Arc::new(users::UserRepositoryForDB::new(conn)));
    }
    Ok(Arc::new(users::UserRepositoryForMemory::new()))
}

// Must not use for production
//const SALT: &str = "Salt should be passed in a more secure way";

// fn new_hash() -> Arc<dyn hash::Hash> {
//     // for now, only 1 implementation
//     Arc::new(hash::HashPbkdf2::new())
// }

// #[cfg(feature = "pbkdf2")]
// fn new_hash() -> hash::HashPbkdf2 {
//     debug!("hash crate is pbkdf2");

//     // for now, only 1 implementation
//     hash::HashPbkdf2::new()
// }

// #[cfg(feature = "scrypt")]
// fn new_hash() -> hash::HashScrypt {
//     debug!("hash crate is scrypt");

//     // for now, only 1 implementation
//     hash::HashScrypt::new()
// }

cfg_if! {
    if #[cfg(feature = "pbkdf2")] {
        fn new_hash() -> hash::HashPbkdf2 {
            debug!("hash crate is pbkdf2");

            // for now, only 1 implementation
            hash::HashPbkdf2::new()
        }
    } else if #[cfg(feature = "scrypt")] {
        fn new_hash() -> hash::HashScrypt {
            debug!("hash crate is scrypt");

            // for now, only 1 implementation
            hash::HashScrypt::new()
        }
     } else {
        compile_error!("One of the features 'pbkdf2' or 'scrypt' must be enabled");
    }
}

fn new_jwt(cjwt: &toml::JWT) -> Arc<dyn jwt::JWT> {
    match cjwt.kind {
        toml::JWTKind::JWTSimple => Arc::new(jwt::SimpleJWT::new(cjwt.duration_min)),
        toml::JWTKind::JsonWebToken => Arc::new(jwt::JsonWebToken::new(cjwt.duration_min * 60)),
        toml::JWTKind::None => Arc::new(jwt::DummyJWT::new()),
    }
}

cfg_if! {
    if #[cfg(feature = "pbkdf2")] {
        pub struct Registry {
            pub conf: toml::Config,
            pub todos_repo: Arc<dyn todos::TodoRepository>,
            pub users_repo: Arc<dyn users::UserRepository>,
            pub jwt: Arc<dyn jwt::JWT>,
            pub hash: hash::HashPbkdf2,
        }
    } else if #[cfg(feature = "scrypt")] {
        pub struct Registry {
            pub conf: toml::Config,
            pub todos_repo: Arc<dyn todos::TodoRepository>,
            pub users_repo: Arc<dyn users::UserRepository>,
            pub jwt: Arc<dyn jwt::JWT>,
            pub hash: hash::HashScrypt,
        }
    } else {
        compile_error!("One of the features 'pbkdf2' or 'scrypt' must be enabled");
    }
}
// pub struct Registry {
//     pub conf: toml::Config,
//     pub todos_repo: Arc<dyn todos::TodoRepository>,
//     pub users_repo: Arc<dyn users::UserRepository>,
//     pub jwt: Arc<dyn jwt::JWT>,
//     pub hash: hash::HashPbkdf2,
// }

impl Registry {
    pub async fn new(conf: toml::Config) -> Result<Self, DbErr> {
        //let db = conf.db.clone();
        let db_conn = new_db_conn(&conf.db).await?;

        let todos_repo = new_todos_repository(db_conn.clone()).await?;
        let users_repo = new_users_repository(db_conn.clone()).await?;
        let hash = new_hash();
        let jwt = new_jwt(&conf.jwt);

        Ok(Self {
            conf,
            todos_repo,
            users_repo,
            jwt,
            hash,
        })
    }

    fn create_auth_usecase(&self) -> Arc<dyn auth::AuthUsecase> {
        Arc::new(auth::AuthAction::new(
            self.users_repo.clone(),
            self.hash.clone(),
            self.jwt.clone(),
        ))
    }

    fn create_admin_usecase(&self) -> Arc<dyn admin::AdminUsecase> {
        // Clone of Arc<T> doesn't have cost because it just increment reference count
        Arc::new(admin::AdminAction::new(
            self.todos_repo.clone(),
            self.users_repo.clone(),
            self.hash.clone(),
        ))
    }

    fn create_app_usecase(&self) -> Arc<dyn app::AppUsecase> {
        // Clone of Arc<T> doesn't have cost because it just increment reference count
        Arc::new(app::AppAction::new(
            self.todos_repo.clone(),
            self.users_repo.clone(),
        ))
    }

    pub fn create_global_state(&self) -> state::GlobalState {
        state::GlobalState {
            app_name: self.conf.app_name.clone(),
        }
    }

    pub fn create_auth_state(&self) -> state::AuthState {
        state::AuthState {
            auth_usecase: self.create_auth_usecase(),
        }
    }

    pub fn create_admin_state(&self) -> state::AdminState {
        state::AdminState {
            admin_usecase: self.create_admin_usecase(),
        }
    }

    pub fn create_app_state(&self) -> state::AppState {
        state::AppState {
            app_usecase: self.create_app_usecase(),
        }
    }
}

use clap::{Parser, Subcommand};
use entities::account::AccountType;
use entities::emails::EmailType;
use entities::{now, EmailEntity};
use regex::Regex;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
};
use utils::database::{EmailAddress, Password};

#[derive(Debug, Parser)]
pub struct RandomUsersCommand {
    #[clap(long, default_value = "10")]
    pub count: usize,
}
impl RandomUsersCommand {
    pub async fn run(&self, database: DatabaseConnection, test_domain: String, default_group: i64) {
        let regex = Regex::new("[a-zA-Z]+").unwrap();
        let client = reqwest::Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::USER_AGENT,
                    reqwest::header::HeaderValue::from_static(
                        "Stalwart Panel Test Tool github.com/wyatt-herkamp/stalwart-panel",
                    ),
                );
                headers.insert(
                    reqwest::header::ACCEPT,
                    reqwest::header::HeaderValue::from_static("application/json"),
                );
                headers
            })
            .build()
            .unwrap();
        let request = client
            .get(format!(
                "https://randomuser.me/api/?results={}&inc=name,email,login&nat=US",
                self.count
            ))
            .build()
            .unwrap();

        let response = client
            .execute(request)
            .await
            .expect("Failed to execute request");
        let response: RandomUserAPIResponse = response.json().await.unwrap();

        for user in response.results {
            if !regex.is_match(&user.name.first) || !regex.is_match(&user.name.last) {
                println!("Skipping user {user:?}");
                continue;
            }
            println!("Creating new User {user:?}");
            let email_address = EmailAddress::new(format!(
                "{}.{}@{}",
                user.name.first, user.name.last, test_domain
            ))
            .expect("Failed to create email address");
            let description = ActiveValue::Set(format!(
                "Password is `{}` `source: randomuser.me`",
                &user.login.password
            ));
            let active_model = entities::account::ActiveModel {
                id: Default::default(),
                password: ActiveValue::Set(
                    Password::new_argon2(&user.login.password).expect("Failed to hash password"),
                ),
                require_password_change: ActiveValue::Set(false),
                quota: ActiveValue::Set(0),
                name: ActiveValue::Set(user.name.into()),
                username: ActiveValue::Set(user.login.username),
                description,
                active: ActiveValue::Set(true),
                backup_email: ActiveValue::Set(Some(user.email)),
                group_id: ActiveValue::Set(default_group),
                account_type: ActiveValue::Set(AccountType::Individual),
                created: now(),
            };

            let account = entities::account::Entity::insert(active_model)
                .on_conflict(
                    OnConflict::column(entities::account::Column::Username)
                        .do_nothing()
                        .to_owned(),
                )
                .exec(&database)
                .await
                .map(|x| x.last_insert_id);
            match account {
                Ok(account) => {
                    if EmailEntity::find()
                        .filter(entities::emails::Column::EmailAddress.eq(email_address.clone()))
                        .count(&database)
                        .await
                        .unwrap()
                        > 0
                    {
                        println!("Email {email_address} already exists");
                        continue;
                    }

                    let active_model = entities::emails::ActiveModel {
                        id: Default::default(),
                        account: ActiveValue::Set(account),
                        email_address: ActiveValue::Set(email_address),
                        email_type: ActiveValue::Set(EmailType::Primary),
                        created: now(),
                    };

                    entities::emails::Entity::insert(active_model)
                        .exec(&database)
                        .await
                        .expect("Failed to insert email");
                }
                Err(err) => {
                    println!("Failed to insert account: {err:?}")
                }
            }
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct RandomUserAPIResponse {
    pub results: Vec<RandomUser>,
    pub info: RandomUserAPIInfo,
}
#[derive(Debug, serde::Deserialize)]
pub struct RandomUser {
    pub name: RandomUserName,
    pub email: EmailAddress,
    pub login: RandomUserLogin,
}
#[derive(Debug, serde::Deserialize)]
pub struct RandomUserLogin {
    pub username: String,
    pub password: String,
}
#[derive(Debug, serde::Deserialize)]
pub struct RandomUserName {
    pub first: String,
    pub last: String,
    pub title: String,
}
impl RandomUserName {
    pub fn is_name_english_alphabet(&self) -> bool {
        self.first
            .chars()
            .all(|x| x.is_ascii_alphabetic() || x == '-')
            && self
                .last
                .chars()
                .all(|x| x.is_ascii_alphabetic() || x == '-')
    }
}

impl Into<String> for RandomUserName {
    fn into(self) -> String {
        format!("{} {}", self.first, self.last)
    }
}
#[derive(Debug, serde::Deserialize)]
pub struct RandomUserAPIInfo {
    pub seed: String,
    pub results: usize,
    pub page: usize,
    pub version: String,
}

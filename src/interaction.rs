use serenity::{
    all::{
        ChannelId, CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
        EditInteractionResponse,
    },
    builder::CreateInteractionResponse,
    http::Http,
    model::{
        prelude::{Interaction, Message},
        user::User,
    },
};

#[trait_variant::make]
pub trait InteractionExtension {
    async fn send_message(&self, http: &Http, content: impl Into<String>) -> serenity::Result<()>;

    async fn send_ephemeral_message(
        &self,
        http: &Http,
        content: impl Into<String>,
    ) -> serenity::Result<()>;

    fn channel_id(&self) -> ChannelId;

    fn message(&self) -> Option<&Message>;

    fn user(&self) -> &User;

    async fn create_response(
        &self,
        http: &Http,
        builder: CreateInteractionResponse,
    ) -> serenity::Result<()>;

    async fn create_followup(
        &self,
        http: &Http,
        builder: CreateInteractionResponseFollowup,
    ) -> serenity::Result<Message>;

    async fn edit_response(
        &self,
        http: &Http,
        builder: EditInteractionResponse,
    ) -> serenity::Result<Message>;

    async fn delete_response(&self, http: &Http) -> serenity::Result<()>;

    async fn defer(&self, http: &Http) -> serenity::Result<()>;
}

impl InteractionExtension for Interaction {
    async fn send_message(&self, http: &Http, content: impl Into<String>) -> serenity::Result<()> {
        let builder = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().content(content),
        );
        match self {
            Interaction::Ping(_ping) => Ok(()),
            Interaction::Autocomplete(_command) => unreachable!(),
            Interaction::Command(command) => command.create_response(http, builder).await,
            Interaction::Component(component) => component.create_response(http, builder).await,
            Interaction::Modal(modal) => modal.create_response(http, builder).await,
            _ => todo!(),
        }
    }

    async fn send_ephemeral_message(
        &self,
        http: &Http,
        content: impl Into<String>,
    ) -> serenity::Result<()> {
        let builder = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(content)
                .ephemeral(true),
        );
        match self {
            Interaction::Ping(_ping) => Ok(()),
            Interaction::Autocomplete(_command) => unreachable!(),
            Interaction::Command(command) => command.create_response(http, builder).await,
            Interaction::Component(component) => component.create_response(http, builder).await,
            Interaction::Modal(modal) => modal.create_response(http, builder).await,
            _ => todo!(),
        }
    }

    fn channel_id(&self) -> ChannelId {
        match self {
            Interaction::Ping(_ping) => unreachable!(),
            Interaction::Command(command) => command.channel_id,
            Interaction::Autocomplete(command) => command.channel_id,
            Interaction::Component(component) => component.channel_id,
            Interaction::Modal(modal) => modal.channel_id,
            _ => todo!(),
        }
    }

    fn message(&self) -> Option<&Message> {
        match self {
            Interaction::Component(component) => Some(&*component.message),
            Interaction::Modal(modal) => modal.message.as_deref(),
            _ => None,
        }
    }

    fn user(&self) -> &User {
        match self {
            Interaction::Ping(_ping) => unreachable!(),
            Interaction::Command(command) => &command.user,
            Interaction::Autocomplete(command) => &command.user,
            Interaction::Component(component) => &component.user,
            Interaction::Modal(modal) => &modal.user,
            _ => todo!(),
        }
    }

    async fn defer(&self, http: &Http) -> serenity::Result<()> {
        match self {
            Interaction::Ping(_ping) => unreachable!(),
            Interaction::Command(command) => command.defer(http).await,
            Interaction::Autocomplete(command) => command.defer(http).await,
            Interaction::Component(component) => component.defer(http).await,
            Interaction::Modal(modal) => modal.defer(http).await,
            _ => todo!(),
        }
    }

    async fn create_response(
        &self,
        http: &Http,
        builder: CreateInteractionResponse,
    ) -> serenity::Result<()> {
        match self {
            Interaction::Ping(_ping) => unreachable!(),
            Interaction::Command(command) => command.create_response(http, builder).await,
            Interaction::Autocomplete(command) => command.create_response(http, builder).await,
            Interaction::Component(component) => component.create_response(http, builder).await,
            Interaction::Modal(modal) => modal.create_response(http, builder).await,
            _ => todo!(),
        }
    }

    async fn create_followup(
        &self,
        http: &Http,
        builder: CreateInteractionResponseFollowup,
    ) -> serenity::Result<Message> {
        match self {
            Interaction::Ping(_ping) => unreachable!(),
            Interaction::Command(command) => command.create_followup(http, builder).await,
            Interaction::Autocomplete(command) => command.create_followup(http, builder).await,
            Interaction::Component(component) => component.create_followup(http, builder).await,
            Interaction::Modal(modal) => modal.create_followup(http, builder).await,
            _ => todo!(),
        }
    }

    async fn edit_response(
        &self,
        http: &Http,
        builder: EditInteractionResponse,
    ) -> serenity::Result<Message> {
        match self {
            Interaction::Ping(_ping) => unreachable!(),
            Interaction::Command(command) => command.edit_response(http, builder).await,
            Interaction::Autocomplete(command) => command.edit_response(http, builder).await,
            Interaction::Component(component) => component.edit_response(http, builder).await,
            Interaction::Modal(modal) => modal.edit_response(http, builder).await,
            _ => todo!(),
        }
    }

    async fn delete_response(&self, http: &Http) -> serenity::Result<()> {
        match self {
            Interaction::Ping(_ping) => unreachable!(),
            Interaction::Command(command) => command.delete_response(http).await,
            Interaction::Autocomplete(command) => command.delete_response(http).await,
            Interaction::Component(component) => component.delete_response(http).await,
            Interaction::Modal(modal) => modal.delete_response(http).await,
            _ => todo!(),
        }
    }
}

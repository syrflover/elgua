use serenity::{
    builder::{CreateInteractionResponse, EditInteractionResponse},
    http::Http,
    model::{
        prelude::{
            interaction::{
                application_command::ApplicationCommandInteraction,
                message_component::MessageComponentInteraction,
            },
            Message,
        },
        user::User,
    },
};

pub enum Interaction<'a> {
    /// Interaction, 실제 상호작용을 할 것인지
    ApplicationCommand(&'a ApplicationCommandInteraction, bool),

    /// Interaction, 실제 상호작용을 할 것인지
    MessageComponent(&'a MessageComponentInteraction, bool),
}

impl<'a> From<&'a ApplicationCommandInteraction> for Interaction<'a> {
    fn from(interaction: &'a ApplicationCommandInteraction) -> Self {
        Self::ApplicationCommand(interaction, true)
    }
}

impl<'a> From<&'a MessageComponentInteraction> for Interaction<'a> {
    fn from(interaction: &'a MessageComponentInteraction) -> Self {
        Self::MessageComponent(interaction, true)
    }
}

impl<'a> From<&'a mut MessageComponentInteraction> for Interaction<'a> {
    fn from(interaction: &'a mut MessageComponentInteraction) -> Self {
        Self::MessageComponent(interaction, true)
    }
}

impl<'n> Interaction<'n> {
    pub fn do_interact(self, do_interact: bool) -> Self {
        match self {
            Self::ApplicationCommand(x, _) => Self::ApplicationCommand(x, do_interact),
            Self::MessageComponent(x, _) => Self::MessageComponent(x, do_interact),
        }
    }

    pub fn user(&self) -> &'n User {
        match self {
            Self::ApplicationCommand(interaction, _) => &interaction.user,
            Self::MessageComponent(interaction, _) => &interaction.user,
        }
    }

    /// Interaction::MessageComponent일 경우에만 Some
    pub fn message(&self) -> Option<&'n Message> {
        match self {
            // Self::ApplicationCommand(interaction, _) => interaction.message
            Self::MessageComponent(interaction, _) => Some(&interaction.message),
            _ => None,
        }
    }

    pub async fn send_message(
        &self,
        http: impl AsRef<Http>,
        m: impl ToString,
    ) -> serenity::Result<()> {
        self.create_interaction_response(http, |resp| {
            resp.interaction_response_data(|message| message.content(m))
        })
        .await

        // match self {
        //     Self::ApplicationCommand(_) => {
        //         self.create_interaction_response(http, |resp| {
        //             resp.interaction_response_data(|message| message.content(m))
        //         })
        //         .await
        //     }

        //     _ => Ok(()),
        // }
    }

    pub async fn create_interaction_response<'a, F>(
        &self,
        http: impl AsRef<Http>,
        f: F,
    ) -> serenity::Result<()>
    where
        for<'b> F:
            FnOnce(&'b mut CreateInteractionResponse<'a>) -> &'b mut CreateInteractionResponse<'a>,
    {
        match self {
            Self::ApplicationCommand(interaction, true) => {
                interaction.create_interaction_response(http, f).await
            }

            Self::MessageComponent(interaction, true) => {
                interaction.create_interaction_response(http, f).await
            }

            _ => Ok(()),
        }
    }

    // 실제 상호작용을 했을 때만 Some(message)를 리턴함
    pub async fn edit_original_interaction_response<F>(
        &self,
        http: impl AsRef<Http>,
        f: F,
    ) -> serenity::Result<Option<Message>>
    where
        F: FnOnce(&mut EditInteractionResponse) -> &mut EditInteractionResponse,
    {
        match self {
            Self::ApplicationCommand(interaction, true) => interaction
                .edit_original_interaction_response(http, f)
                .await
                .map(Some),

            Self::MessageComponent(interaction, true) => interaction
                .edit_original_interaction_response(http, f)
                .await
                .map(Some),

            _ => Ok(None),
        }
    }
}

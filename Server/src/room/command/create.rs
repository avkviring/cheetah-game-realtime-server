use cheetah_relay_common::commands::command::load::CreateGameObjectCommand;
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::constants::ClientId;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use cheetah_relay_common::room::object::ClientGameObjectId;
use cheetah_relay_common::udp::protocol::frame::applications::ApplicationCommand::S2CCommandWithMeta;

use crate::room::{GameObjectCreateErrors, Room};
use crate::room::command::{CommandContext, error_c2s_command, ServerRoomCommandExecutor, trace_c2s_command};
use crate::room::object::GameObject;
use crate::room::object::id::ServerGameObjectId;

impl Room {
    pub fn create_game_object(&mut self,
                              object_id: &ClientGameObjectId,
                              template: u16,
                              access_groups: AccessGroups,
                              fields: GameObjectFields,
                              context: &CommandContext,
    ) -> Result<&GameObject, GameObjectCreateErrors> {
        let id = ServerGameObjectId::new(context.current_client.map(|c| c.configuration.id), object_id);
        if self.objects.get(&id).is_some() {
            Result::Err(GameObjectCreateErrors::AlreadyExists(id))
        } else {
            let mut object = GameObject::new(
                id.clone(),
                template,
                access_groups.clone(),
                fields.clone(),
                self.collector.clone(),
            );

            object.send_to_clients(context, |client_id, object_id| {
                S2CCommandUnion::Create(CreateGameObjectCommand {
                    object_id,
                    template,
                    access_groups: access_groups.clone(),
                    fields: fields.clone(),
                })
            });


            self.objects.insert(id.clone(), object);
            Result::Ok(self.objects.get(&id).unwrap())
        }
    }
}


impl ServerRoomCommandExecutor for CreateGameObjectCommand {
    fn execute(self, room: &mut Room, context: &CommandContext) {
        let client = context.current_client.unwrap();
        if self.access_groups.is_sub_groups(&client.configuration.groups) {
            let id = self.object_id;
            match room.create_game_object(&id, self.template, self.access_groups, self.fields, context) {
                Ok(_) => {
                    trace_c2s_command(
                        "LoadGameObject",
                        room,
                        client,
                        format!("Object created with id {:?}", id),
                    );
                }
                Err(_) => {
                    error_c2s_command(
                        "LoadGameObject",
                        room,
                        client,
                        format!("Object already exists with id {:?}", id),
                    );
                }
            }
        } else {
            error_c2s_command(
                "LoadGameObject",
                room,
                client,
                format!("Incorrect access group {:?} with client groups {:?}", self.access_groups, client.configuration.groups),
            );
        };
    }
}

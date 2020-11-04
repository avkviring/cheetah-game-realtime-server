use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;

use crate::room::command::{CommandContext, ServerRoomCommandExecutor};
use crate::room::object::GameObject;
use crate::room::object::id::ServerGameObjectId;
use crate::room::Room;

impl Room {
    pub fn delete_game_object(&mut self, id: &ServerGameObjectId, context: &CommandContext) {
        let object: Option<&mut GameObject> = self.objects.get_mut(id);
        match object {
            None => {
                log::error!("game object not found {:?}", id)
            }
            Some(object) => {
                object.send_to_clients(context, |_, object_id| {
                    S2CCommandUnion::Delete(DeleteGameObjectCommand { object_id })
                });
                self.objects.remove(id);
            }
        }
    }
}


impl ServerRoomCommandExecutor for DeleteGameObjectCommand {
    fn execute(self, room: &mut Room, context: &CommandContext) {
        let result = room.get_object_with_check_access(context.current_client.unwrap(), &self.object_id);
        if let Some(object) = result {
            let id = &object.id.clone();
            room.delete_game_object(id, context);
        }
    }
}


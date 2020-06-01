use std::cell::RefCell;
use std::rc::Rc;

use cheetah_relay::network::s2c::{AffectedClients, S2CCommandCollector, S2CCommandUnion};
use cheetah_relay::room::clients::Clients;
use cheetah_relay::room::objects::object::GameObject;
use cheetah_relay::room::room::Room;
use cheetah_relay_common::constants::ClientId;
use cheetah_relay_common::network::command::event::EventCommand;
use cheetah_relay_common::network::command::float_counter::SetFloatCounterCommand;
use cheetah_relay_common::network::command::long_counter::SetLongCounterCommand;
use cheetah_relay_common::network::command::structure::SetStructCommand;
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::network::command::upload::UploadGameObjectS2CCommand;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;

use crate::unit::relay::room::clients::client_stub_with_access_group;
use crate::unit::relay::room::room::room_stub;
use crate::unit::relay::room::setup_client;

#[test]
fn test_affects_client() {
    let groups = AccessGroups::from(0b111);
    let mut clients = Clients::default();
    clients
        .clients
        .insert(0, Rc::new(client_stub_with_access_group(0, 0b1)));
    clients
        .clients
        .insert(1, Rc::new(client_stub_with_access_group(1, 0b100000)));
    clients
        .clients
        .insert(2, Rc::new(client_stub_with_access_group(2, 0b111)));

    let affected_client = AffectedClients::new_from_clients(&clients, &groups);
    assert_eq!(affected_client.clients.contains(&0), true);
    assert_eq!(affected_client.clients.contains(&1), false);
    assert_eq!(affected_client.clients.contains(&2), true);
}

#[test]
fn should_s2c_collect_on_object_create() {
    let (mut room, collector) = setup();
    let client = setup_client(&mut room, "HASH", 0b100);
    room.create_root_game_object(10, AccessGroups::from(0b100), Default::default())
        .unwrap();

    let id = client.configuration.id;
    assert_command(
        collector,
        id,
        S2CCommandUnion::UploadGameObject(UploadGameObjectS2CCommand {
            id: 10,
            fields: Default::default(),
        }),
    );
}

#[test]
fn should_s2c_collect_on_client_connect() {
    let (mut room, commands) = setup();
    room.create_root_game_object(10, AccessGroups::from(0b100), GameObjectFields::default());
    room.create_root_game_object(11, AccessGroups::from(0b100), GameObjectFields::default());
    room.create_root_game_object(9, AccessGroups::from(0b100), GameObjectFields::default());
    room.create_root_game_object(1, AccessGroups::from(0b100), GameObjectFields::default());

    let client = setup_client(&mut room, "HASH", 0b100);
    let id = client.configuration.id;
    assert_command(
        commands.clone(),
        id,
        S2CCommandUnion::UploadGameObject(UploadGameObjectS2CCommand {
            id: 10,
            fields: Default::default(),
        }),
    );

    assert_command(
        commands,
        id,
        S2CCommandUnion::UploadGameObject(UploadGameObjectS2CCommand {
            id: 11,
            fields: Default::default(),
        }),
    );
}

#[test]
fn should_s2c_collect_on_client_disconnect() {
    let (mut room, commands) = setup();
    let client_a = setup_client(&mut room, "HASH_A", 0b100);
    room.create_client_game_object(
        &client_a,
        1,
        AccessGroups::from(0b100),
        GameObjectFields::default(),
    )
    .unwrap();
    room.create_client_game_object(
        &client_a,
        2,
        AccessGroups::from(0b100),
        GameObjectFields::default(),
    )
    .unwrap();
    room.create_client_game_object(
        &client_a,
        3,
        AccessGroups::from(0b100),
        GameObjectFields::default(),
    )
    .unwrap();

    let client_b = setup_client(&mut room, "HASH_B", 0b100);
    clear_commands(commands.clone(), client_b.configuration.id);
    room.client_disconnect(&client_a);

    assert_command(
        commands.clone(),
        client_b.configuration.id,
        S2CCommandUnion::UnloadGameObject(UnloadGameObjectCommand {
            global_object_id: GameObject::get_global_object_id_by_client(&client_a, 1),
        }),
    );

    assert_command(
        commands.clone(),
        client_b.configuration.id,
        S2CCommandUnion::UnloadGameObject(UnloadGameObjectCommand {
            global_object_id: GameObject::get_global_object_id_by_client(&client_a, 2),
        }),
    );

    assert_command(
        commands.clone(),
        client_b.configuration.id,
        S2CCommandUnion::UnloadGameObject(UnloadGameObjectCommand {
            global_object_id: GameObject::get_global_object_id_by_client(&client_a, 3),
        }),
    );
}

#[test]
fn should_s2c_collect_on_update_long_counter() {
    let (mut room, commands) = setup();
    let client = setup_client(&mut room, "HASH_A", 0b100);
    let id = room
        .create_client_game_object(
            &client,
            1,
            AccessGroups::from(0b100),
            GameObjectFields::default(),
        )
        .ok()
        .unwrap();
    let object = room.objects.get(id).unwrap();
    let object = &*object;
    room.object_increment_long_counter(&mut object.borrow_mut(), 1, 155);
    clear_commands(commands.clone(), client.configuration.id);
    room.object_increment_long_counter(&mut object.borrow_mut(), 1, 55);

    assert_command(
        commands,
        client.configuration.id,
        S2CCommandUnion::SetLongCounter(SetLongCounterCommand {
            global_object_id: id,
            field_id: 1,
            value: 210,
        }),
    );
}

#[test]
fn should_s2c_collect_on_update_float_counter() {
    let (mut room, commands) = setup();
    let client = setup_client(&mut room, "HASH_A", 0b100);
    let id = room
        .create_client_game_object(
            &client,
            1,
            AccessGroups::from(0b100),
            GameObjectFields::default(),
        )
        .ok()
        .unwrap();
    let object = room.objects.get(id).unwrap();
    let object = object.clone();
    let object = &*object;
    room.object_increment_float_counter(&mut object.borrow_mut(), 1, 155.0);
    clear_commands(commands.clone(), client.configuration.id);
    room.object_increment_float_counter(&mut object.borrow_mut(), 1, 55.0);

    assert_command(
        commands,
        client.configuration.id,
        S2CCommandUnion::SetFloatCounter(SetFloatCounterCommand {
            global_object_id: id,
            field_id: 1,
            value: 210.0,
        }),
    );
}

#[test]
fn should_s2c_collect_on_fire_event() {
    let (mut room, commands) = setup();
    let client = setup_client(&mut room, "HASH_A", 0b100);
    let id = room
        .create_client_game_object(
            &client,
            1,
            AccessGroups::from(0b100),
            GameObjectFields::default(),
        )
        .ok()
        .unwrap();
    let object = room.objects.get(id).unwrap();
    let object = object.clone();
    let object = &*object;
    clear_commands(commands.clone(), client.configuration.id);
    room.object_send_event(&mut object.borrow_mut(), 10, &vec![1, 2, 3, 4, 5]);

    assert_command(
        commands,
        client.configuration.id,
        S2CCommandUnion::Event(EventCommand {
            global_object_id: id,
            field_id: 10,
            event: vec![1, 2, 3, 4, 5],
        }),
    );
}

#[test]
fn should_s2c_collect_on_update_struct() {
    let (mut room, commands) = setup();
    let client = setup_client(&mut room, "HASH_A", 0b100);
    let id = room
        .create_client_game_object(
            &client,
            1,
            AccessGroups::from(0b100),
            GameObjectFields::default(),
        )
        .ok()
        .unwrap();
    let object = room.objects.get(id).unwrap();
    let object = object.clone();
    let object = &*object;
    clear_commands(commands.clone(), client.configuration.id);
    room.object_update_struct(&mut object.borrow_mut(), 10, vec![1, 2, 3, 4, 5]);

    assert_command(
        commands,
        client.configuration.id,
        S2CCommandUnion::SetStruct(SetStructCommand {
            global_object_id: id,
            field_id: 10,
            data: vec![1, 2, 3, 4, 5],
        }),
    );
}

fn clear_commands(collector: Rc<RefCell<S2CCommandCollector>>, client_id: ClientId) {
    let mut collector = collector.borrow_mut();
    collector
        .commands_by_client
        .get_mut(&client_id)
        .map(|f| f.clear());
}

fn assert_command(
    collector: Rc<RefCell<S2CCommandCollector>>,
    client_id: ClientId,
    expected: S2CCommandUnion,
) {
    let mut collector = collector.borrow_mut();
    let commands = collector.commands_by_client.get_mut(&client_id).unwrap();
    assert_eq!(commands.len() > 0, true);
    let actual = commands.pop_front().unwrap();
    assert_eq!(actual, expected)
}

fn setup() -> (Room, Rc<RefCell<S2CCommandCollector>>) {
    let mut room = room_stub();
    let collector = Rc::new(RefCell::new(S2CCommandCollector::new()));
    room.listener.add_listener(collector.clone());
    (room, collector)
}

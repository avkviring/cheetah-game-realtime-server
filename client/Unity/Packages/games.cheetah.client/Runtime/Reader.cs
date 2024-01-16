using System;
using System.Collections.Generic;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Types.Command;
using Games.Cheetah.Client.Types.Field;
using Games.Cheetah.Client.Types.Object;
using Unity.Collections;

namespace Games.Cheetah.Client
{
    public class Reader
    {
        private readonly NetworkClient client;
        private readonly CodecRegistry codecRegistry;
        private readonly Dictionary<NetworkObjectId, ushort> templateByObject = new();
        private readonly Dictionary<NetworkObjectId, ushort> templateByDeletedObject = new();
        private readonly Dictionary<NetworkObjectId, NetworkObjectConstructor> creatingObjects = new();
        private readonly Dictionary<NetworkObjectId, NetworkObjectConstructor> createdObjectsInUpdate = new();

        public Reader(NetworkClient client, CodecRegistry codecRegistry)
        {
            this.client = client;
            this.codecRegistry = codecRegistry;
        }

        /**
         * Флаг, создавались ли объекты в текущем Update.
         * Можно проверять, чтобы не выделять память каждый кадр
         */
        public bool HasCreateObjectsInCurrentUpdate => createdObjectsInUpdate.Count > 0;

        /**
         * Получить объекты с сервера, создание которых завершилось в текущем Update
         */
        public void CollectCreatedObjectsInCurrentUpdate(ushort template, IList<NetworkObjectConstructor> result)
        {
            foreach (var (key, value) in createdObjectsInUpdate)
            {
                if (value.NetworkObject.Template == template)
                {
                    result.Add(value);
                }
            }
        }

        [Obsolete]
        public IList<NetworkObjectConstructor> GetCreatedObjectsInCurrentUpdate(ushort template)
        {
            var result = new List<NetworkObjectConstructor>();
            CollectCreatedObjectsInCurrentUpdate(template, result);
            return result;
        }

        /**
         * Получить список новых игроков полученных в текущем Update. Текущий клиент может получить такой список если только он в состоянии Attached к комнате.
         * Игроки подключенные до того как текущий игрок подключился не попадают в данный список.
         */
        public NativeList<ushort> GetConnectedMemberInUpdate()
        {
            var result = new NativeList<ushort>(sbyte.MaxValue, Allocator.TempJob);
            for (var i = 0; i < client.S2CCommandsCount; i++)
            {
                ref var command = ref client.s2cCommands[i];
                if (command.commandType != CommandType.MemberConnected) continue;
                ref var memberConnectedCommand = ref command.commandUnion.memberConnected;
                result.Add(memberConnectedCommand.MemberId);
            }

            return result;
        }

        /**
         * Получить список вышедших из игры игроков. Текущий клиент должен быть в состоянии Attached к комнате.
         * Все вышедшие игроки до тех пор пока текущий клиент не был в состоянии Attached в данный список не попадают.
         */
        public NativeList<ushort> GetDisconnectedMemberInUpdate()
        {
            var result = new NativeList<ushort>(sbyte.MaxValue, Allocator.TempJob);
            for (var i = 0; i < client.S2CCommandsCount; i++)
            {
                ref var command = ref client.s2cCommands[i];
                if (command.commandType != CommandType.MemberDisconnected) continue;
                ref var memberDisconnected = ref command.commandUnion.memberDisconnected;
                result.Add(memberDisconnected.MemberId);
            }

            return result;
        }


        /**
         * Получить список изменений double полей объекта в текущем цикле. Если для одного поля было несколько изменений - вовзвращается последнее.
         */
        public NativeList<(NetworkObjectId, double)> GetModifiedDoubles(FieldId.Double fieldId)
        {
            var result = new NativeList<(NetworkObjectId, double)>(sbyte.MaxValue, Allocator.TempJob);
            for (var i = 0; i < client.S2CCommandsCount; i++)
            {
                ref var command = ref client.s2cCommands[i];
                if (command.commandType != CommandType.SetDouble) continue;
                ref var setCommand = ref command.commandUnion.setDouble;
                var commandObjectId = setCommand.objectId;

                if (IsCreatingObject(commandObjectId))
                {
                    continue;
                }

                if (FilterCommand(fieldId, setCommand.fieldId))
                {
                    result.Add((commandObjectId, setCommand.value));
                }
            }

            return result;
        }

        private bool FilterCommand(FieldId fieldId, ushort commandFieldId)
        {
            return commandFieldId == fieldId.Id;
        }

        /**
         * Получить список изменений long полей объекта в текущем цикле. Если для одного поля было несколько изменений - вовзвращается последнее.
         */
        public NativeList<(NetworkObjectId, long)> GetModifiedLongs(FieldId.Long fieldId)
        {
            var result = new NativeList<(NetworkObjectId, long)>(sbyte.MaxValue, Allocator.TempJob);
            for (var i = 0; i < client.S2CCommandsCount; i++)
            {
                ref var command = ref client.s2cCommands[i];
                if (command.commandType != CommandType.SetLong) continue;
                ref var setLongCommand = ref command.commandUnion.setLong;
                var commandObjectId = setLongCommand.objectId;

                if (IsCreatingObject(commandObjectId))
                {
                    continue;
                }

                if (FilterCommand(fieldId, setLongCommand.fieldId))
                {
                    result.Add((commandObjectId, setLongCommand.value));
                }
            }

            return result;
        }


        /**
         * Получить список изменений structure полей объекта в текущем цикле. Если для одного поля было несколько изменений - вовзвращается последнее.
         */
        public NativeList<(NetworkObjectId, T)> GetModifiedStructures<T>(FieldId.Structure fieldId)
            where T : unmanaged
        {
            var result = new NativeList<(NetworkObjectId, T)>(sbyte.MaxValue, Allocator.TempJob);
            for (var i = 0; i < client.S2CCommandsCount; i++)
            {
                ref var command = ref client.s2cCommands[i];
                if (command.commandType != CommandType.SetStructure) continue;
                ref var setCommand = ref command.commandUnion.binaryField;
                var commandObjectId = setCommand.objectId;

                if (IsCreatingObject(commandObjectId))
                {
                    continue;
                }

                if (FilterCommand(fieldId, setCommand.fieldId))
                {
                    var item = new T();
                    var networkBuffer = setCommand.value;
                    codecRegistry.GetCodec<T>().Decode(ref networkBuffer, ref item);
                    result.Add((commandObjectId, item));
                }
            }

            return result;
        }

        /**
       * Получить список добавленных элементов.
       */
        public NativeList<(NetworkObjectId, T)> GetAddedItems<T>(FieldId.Items fieldId)
            where T : unmanaged
        {
            var result = new NativeList<(NetworkObjectId, T)>(sbyte.MaxValue, Allocator.TempJob);
            for (var i = 0; i < client.S2CCommandsCount; i++)
            {
                ref var command = ref client.s2cCommands[i];
                if (command.commandType != CommandType.AddItem) continue;
                ref var binaryField = ref command.commandUnion.binaryField;
                var commandObjectId = binaryField.objectId;

                if (IsCreatingObject(commandObjectId))
                {
                    continue;
                }

                if (!FilterCommand(fieldId, binaryField.fieldId)) continue;

                var item = new T();
                var networkBuffer = binaryField.value;
                codecRegistry.GetCodec<T>().Decode(ref networkBuffer, ref item);
                result.Add((commandObjectId, item));
            }

            return result;
        }

        public void CollectModifiedStructures<T>(FieldId.Structure fieldId, List<(NetworkObjectId, T)> structures)
            where T : new()
        {
            for (var i = 0; i < client.S2CCommandsCount; i++)
            {
                ref var command = ref client.s2cCommands[i];
                if (command.commandType != CommandType.SetStructure) continue;
                ref var setCommand = ref command.commandUnion.binaryField;
                var commandObjectId = setCommand.objectId;

                if (IsCreatingObject(commandObjectId))
                {
                    continue;
                }

                if (FilterCommand(fieldId, setCommand.fieldId))
                {
                    var item = new T();
                    var networkBuffer = setCommand.value;
                    codecRegistry.GetCodec<T>().Decode(ref networkBuffer, ref item);
                    structures.Add((commandObjectId, item));
                }
            }
        }

        /**
             * Получить список событий по объекту в текущем цикле.
             */
        public NativeList<(NetworkObjectId, T)> GetEvents<T>(FieldId.Event eventId) where T : unmanaged
        {
            var result = new NativeList<(NetworkObjectId, T)>(sbyte.MaxValue, Allocator.TempJob);
            for (var i = 0; i < client.S2CCommandsCount; i++)
            {
                ref var command = ref client.s2cCommands[i];
                if (command.commandType != CommandType.SendEvent) continue;
                var commandObjectId = command.commandUnion.setEvent.objectId;
                ref var eventCommand = ref command.commandUnion.setEvent;

                if (FilterCommand(eventId, eventCommand.fieldId))
                {
                    var item = new T();
                    var networkBuffer = eventCommand.eventData;
                    codecRegistry.GetCodec<T>().Decode(ref networkBuffer, ref item);
                    result.Add((commandObjectId, item));
                }
            }

            return result;
        }

        public void CollectEvents<T>(FieldId.Event eventId, List<(NetworkObjectId, T)> events) where T : new()
        {
            for (var i = 0; i < client.S2CCommandsCount; i++)
            {
                ref var command = ref client.s2cCommands[i];
                if (command.commandType != CommandType.SendEvent) continue;
                var commandObjectId = command.commandUnion.setEvent.objectId;
                ref var eventCommand = ref command.commandUnion.setEvent;
                if (FilterCommand(eventId, eventCommand.fieldId))
                {
                    var item = new T();
                    var networkBuffer = eventCommand.eventData;
                    codecRegistry.GetCodec<T>().Decode(ref networkBuffer, ref item);
                    events.Add((commandObjectId, item));
                }
            }
        }


        /**
         * Получить список удаленных объектов в текущем Update.
         */
        public NativeParallelHashSet<NetworkObjectId> GetDeletedObjects(ushort template)
        {
            var result = new NativeParallelHashSet<NetworkObjectId>(sbyte.MaxValue, Allocator.TempJob);
            for (var i = 0; i < client.S2CCommandsCount; i++)
            {
                ref var command = ref client.s2cCommands[i];
                if (command.commandType != CommandType.DeleteObject) continue;
                var commandObjectId = command.commandUnion.deleteField.objectId;
                if (TryGetTemplate(commandObjectId, out var objectTemplate) && objectTemplate == template)
                {
                    result.Add(commandObjectId);
                }
            }

            return result;
        }


        /**
         * Получить список удаленных полей объекта в текущем Update.
         */
        public NativeList<S2CCommands.DeleteField> GetDeleteFields(ushort template, FieldId fieldId)
        {
            var result = new NativeList<S2CCommands.DeleteField>(sbyte.MaxValue, Allocator.TempJob);
            for (var i = 0; i < client.S2CCommandsCount; i++)
            {
                ref var command = ref client.s2cCommands[i];
                if (command.commandType != CommandType.DeleteField) continue;
                ref var commandDeleteField = ref command.commandUnion.deleteField;
                ref var deleteFieldObjectId = ref commandDeleteField.objectId;

                if (TryGetTemplate(deleteFieldObjectId, out var objectTemplate) && objectTemplate == template &&
                    commandDeleteField.fieldId ==
                    fieldId.Id && commandDeleteField.fieldType == fieldId.Type)
                {
                    result.Add(commandDeleteField);
                }
            }

            return result;
        }

        public void Update()
        {
            templateByDeletedObject.Clear();
            createdObjectsInUpdate.Clear();
            ProcessObjectCommands();
        }


        private void ProcessObjectCommands()
        {
            for (var i = 0; i < client.S2CCommandsCount; i++)
            {
                ref var command = ref client.s2cCommands[i];
                switch (command.commandType)
                {
                    case CommandType.CreateGameObject:
                        OnObjectCreate(command);
                        break;
                    case CommandType.CreatedGameObject:
                        OnObjectCreated(command);
                        break;
                    case CommandType.DeleteObject:
                        OnObjectDeleted(command);
                        break;
                    case CommandType.SetLong:
                        OnSetLong(command);
                        break;
                    case CommandType.SetDouble:
                        OnSetDouble(command);
                        break;
                    case CommandType.SetStructure:
                        OnSetStructure(command);
                        break;
                }
            }
        }


        private void OnObjectCreate(S2CCommand command)
        {
            var createNetworkObjectId = command.commandUnion.createObject.objectId;
            var template = command.commandUnion.createObject.template;
            var networkObject = new NetworkObject(createNetworkObjectId, template);

            templateByObject[createNetworkObjectId] = template;
            creatingObjects[createNetworkObjectId] = new NetworkObjectConstructor(networkObject, codecRegistry);
        }

        private void OnObjectCreated(S2CCommand command)
        {
            var networkObjectId = command.commandUnion.createdObject.objectId;
            createdObjectsInUpdate.Add(networkObjectId, creatingObjects[networkObjectId]);
            creatingObjects.Remove(networkObjectId);
        }

        private void OnObjectDeleted(S2CCommand command)
        {
            var objectId = command.commandUnion.deleteObject.objectId;
            creatingObjects.Remove(objectId);
            var template = templateByObject[objectId];
            templateByObject.Remove(objectId);
            templateByDeletedObject.Add(objectId, template);
        }

        private void OnSetLong(S2CCommand command)
        {
            ref var setField = ref command.commandUnion.setLong;
            if (creatingObjects.TryGetValue(setField.objectId, out var constructor))
            {
                constructor.longs[setField.fieldId] = setField.value;
            }
        }

        private void OnSetDouble(S2CCommand command)
        {
            ref var setField = ref command.commandUnion.setDouble;
            if (creatingObjects.TryGetValue(setField.objectId, out var constructor))
            {
                constructor.doubles[setField.fieldId] = setField.value;
            }
        }

        private void OnSetStructure(S2CCommand command)
        {
            ref var setField = ref command.commandUnion.binaryField;
            if (creatingObjects.TryGetValue(setField.objectId, out var constructor))
            {
                constructor.structures[setField.fieldId] = setField.value;
            }
        }


        public void RegisterSelfObject(NetworkObjectId objectId, ushort template)
        {
            templateByObject[objectId] = template;
        }

        private bool TryGetTemplate(NetworkObjectId networkObjectId, out ushort template)
        {
            if (templateByObject.TryGetValue(networkObjectId, out template))
            {
                return true;
            }

            if (templateByDeletedObject.TryGetValue(networkObjectId, out template))
            {
                return true;
            }

            template = 0;
            return false;
        }

        private bool IsCreatingObject(NetworkObjectId objectId)
        {
            return creatingObjects.ContainsKey(objectId);
        }
    }
}
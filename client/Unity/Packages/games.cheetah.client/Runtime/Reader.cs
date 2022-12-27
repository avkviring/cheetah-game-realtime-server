using System;
using System.Collections.Generic;
using System.Linq;
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

        public IList<NetworkObjectConstructor> GetCreatedObjectsInCurrentUpdate(ushort template)
        {
            return createdObjectsInUpdate
                .Where(it => it.Value.NetworkObject.Template == template)
                .Select(it => it.Value)
                .ToList();
        }

        public NativeList<ushort> GetConnectedMemberInUpdate()
        {
            var result = new NativeList<ushort>(sbyte.MaxValue, Allocator.TempJob);
            for (var i = 0; i < client.s2cCommandsCount; i++)
            {
                ref var command = ref NetworkClient.s2cCommands[i];
                if (command.commandType != CommandType.MemberConnected) continue;
                ref var memberConnectedCommand = ref command.commandUnion.memberConnected;
                result.Add(memberConnectedCommand.MemberId);
            }

            return result;
        }


        public NativeParallelHashMap<NetworkObjectId, double> GetModifiedDoubles(ushort template, FieldId.Double fieldId)
        {
            var result = new NativeParallelHashMap<NetworkObjectId, double>(sbyte.MaxValue, Allocator.TempJob);
            for (var i = 0; i < client.s2cCommandsCount; i++)
            {
                ref var command = ref NetworkClient.s2cCommands[i];
                if (command.commandType != CommandType.SetDouble) continue;
                ref var setCommand = ref command.commandUnion.setDouble;
                var commandObjectId = setCommand.objectId;
                if (GetTemplate(commandObjectId) == template && setCommand.fieldId == fieldId.Id)
                {
                    result[commandObjectId] = setCommand.value;
                }
            }

            return result;
        }

        public NativeParallelHashMap<NetworkObjectId, long> GetModifiedLongs(ushort template, FieldId.Long fieldId)
        {
            var result = new NativeParallelHashMap<NetworkObjectId, long>(sbyte.MaxValue, Allocator.TempJob);
            for (var i = 0; i < client.s2cCommandsCount; i++)
            {
                ref var command = ref NetworkClient.s2cCommands[i];
                if (command.commandType != CommandType.SetLong) continue;
                ref var setCommand = ref command.commandUnion.setLong;
                var commandObjectId = setCommand.objectId;
                if (GetTemplate(commandObjectId) == template && setCommand.fieldId == fieldId.Id)
                {
                    result[commandObjectId] = setCommand.value;
                }
            }

            return result;
        }


        public NativeParallelHashMap<NetworkObjectId, T> GetModifiedStructures<T>(ushort template, FieldId.Structure fieldId)
            where T : unmanaged
        {
            var result = new NativeParallelHashMap<NetworkObjectId, T>(sbyte.MaxValue, Allocator.TempJob);
            for (var i = 0; i < client.s2cCommandsCount; i++)
            {
                ref var command = ref NetworkClient.s2cCommands[i];
                if (command.commandType != CommandType.SetStructure) continue;
                ref var setCommand = ref command.commandUnion.setStructure;
                var commandObjectId = setCommand.objectId;
                if (GetTemplate(commandObjectId) == template && setCommand.fieldId == fieldId.Id)
                {
                    var item = new T();
                    var networkBuffer = setCommand.value;
                    codecRegistry.GetCodec<T>().Decode(ref networkBuffer, ref item);
                    result[commandObjectId] = item;
                }
            }

            return result;
        }

        public NativeParallelHashMap<NetworkObjectId, T> GetEvents<T>(ushort template, FieldId.Event eventId) where T : struct
        {
            var result = new NativeParallelHashMap<NetworkObjectId, T>(sbyte.MaxValue, Allocator.TempJob);
            for (var i = 0; i < client.s2cCommandsCount; i++)
            {
                ref var command = ref NetworkClient.s2cCommands[i];
                if (command.commandType != CommandType.SendEvent) continue;
                var networkObjectId = command.commandUnion.setEvent.objectId;
                ref var eventCommand = ref command.commandUnion.setEvent;
                if (GetTemplate(networkObjectId) == template && eventCommand.fieldId == eventId.Id)
                {
                    var item = new T();
                    var networkBuffer = eventCommand.eventData;
                    codecRegistry.GetCodec<T>().Decode(ref networkBuffer, ref item);
                    result[networkObjectId] = item;
                }
            }

            return result;
        }


        public NativeParallelHashSet<NetworkObjectId> GetDeletedObjects(ushort template)
        {
            var result = new NativeParallelHashSet<NetworkObjectId>(sbyte.MaxValue, Allocator.TempJob);
            for (var i = 0; i < client.s2cCommandsCount; i++)
            {
                ref var command = ref NetworkClient.s2cCommands[i];
                if (command.commandType != CommandType.DeleteObject) continue;
                var commandObjectId = command.commandUnion.deleteField.objectId;
                if (GetTemplate(commandObjectId) == template)
                {
                    result.Add(commandObjectId);
                }
            }

            return result;
        }


        public NativeList<S2CCommands.DeleteField> GetDeleteFields(ushort template, FieldId fieldId)
        {
            var result = new NativeList<S2CCommands.DeleteField>(sbyte.MaxValue, Allocator.TempJob);
            for (var i = 0; i < client.s2cCommandsCount; i++)
            {
                ref var command = ref NetworkClient.s2cCommands[i];
                if (command.commandType != CommandType.DeleteField) continue;
                ref var commandDeleteField = ref command.commandUnion.deleteField;
                ref var deleteFieldObjectId = ref commandDeleteField.objectId;
                if (GetTemplate(deleteFieldObjectId) == template && commandDeleteField.fieldId ==
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
            for (var i = 0; i < client.s2cCommandsCount; i++)
            {
                ref var command = ref NetworkClient.s2cCommands[i];
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
            ref var setField = ref command.commandUnion.setStructure;
            if (creatingObjects.TryGetValue(setField.objectId, out var constructor))
            {
                constructor.structures[setField.fieldId] = setField.value;
            }
        }


        public void RegisterSelfObject(NetworkObjectId objectId, ushort template)
        {
            templateByObject[objectId] = template;
        }

        private ushort GetTemplate(NetworkObjectId networkObjectId)
        {
            if (templateByObject.TryGetValue(networkObjectId, out var template))
            {
                return template;
            }

            if (templateByDeletedObject.TryGetValue(networkObjectId, out var templateDeletedObject))
            {
                return templateDeletedObject;
            }

            throw new Exception("NetworkObject with id = " + networkObjectId + " not created");
        }
    }
}
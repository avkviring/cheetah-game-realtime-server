using System.Collections.Generic;
using System.Linq;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Types.Command;
using Games.Cheetah.Client.Types.Field;
using Games.Cheetah.Client.Types.Network;
using Games.Cheetah.Client.Types.Object;

namespace Games.Cheetah.Client.Internal
{
    public class FFIMock : IFFI, INetworkClientMock

    {
        private readonly CodecRegistry codecRegistry;
        private ushort clientIdGenerator;
        private Dictionary<ObjectFieldId, object> fields = new();
        private Dictionary<ObjectFieldId, Queue<EventPayload>> events = new();
        private readonly List<S2CCommand> s2cCommands = new();
        private readonly Dictionary<NetworkObjectId, NetworkObject> creatingObjects = new();
        private readonly Dictionary<NetworkObjectId, NetworkObject> createdObjects = new();
        private ushort memberId;
        private uint idGenerator;

        record ObjectFieldId
        {
            internal FieldId fieldId;
            internal NetworkObjectId objectId;
        };

        record EventPayload
        {
            internal object eventData;
            internal ushort? targetUserOpt;
        }

        public FFIMock(CodecRegistry codecRegistry)
        {
            this.codecRegistry = codecRegistry;
        }


        public byte CreateClient(ulong connectionId, string serverAddress, ushort memberId, ulong roomId, ref NetworkBuffer userPrivateKey,
            ulong disconnectTimeInSec, out ushort clientId)
        {
            clientIdGenerator++;
            clientId = clientIdGenerator;
            return 0;
        }

        public byte GetConnectionStatus(ushort clientId, out ConnectionStatus status)
        {
            status = ConnectionStatus.Connected;
            return 0;
        }

        public byte GetStatistics(ushort clientId, out Statistics clientStatistics)
        {
            clientStatistics = new Statistics();
            return 0;
        }

        public unsafe byte Receive(ushort clientId, S2CCommand* commands, ref ushort count)
        {
            var i = 0;
            foreach (var s2CCommand in s2cCommands)
            {
                commands[i] = s2CCommand;
                i++;
            }

            count = (byte)s2cCommands.Count;
            s2cCommands.Clear();
            return 0;
        }

        public byte DestroyClient(ushort clientId)
        {
            return 0;
        }

        public byte AttachToRoom(ushort clientId)
        {
            return 0;
        }

        public byte DetachFromRoom(ushort clientId)
        {
            return 0;
        }

        public byte SetChannelType(ushort clientId, ReliabilityGuarantees reliabilityGuarantees, byte group)
        {
            return 0;
        }

        public byte SetRttEmulation(ushort clientId, ulong rttInMs, double rttDispersion)
        {
            return 0;
        }

        public byte SetDropEmulation(ushort clientId, double dropProbability, ulong dropTimeInMs)
        {
            return 0;
        }

        public byte ResetEmulation(ushort clientId)
        {
            return 0;
        }

        public void GetLastErrorMsg(ref NetworkBuffer buffer)
        {
        }

        public byte GetServerTime(ushort clientId, out ulong time)
        {
            time = 0;
            return 0;
        }

        public byte Set(ushort clientId, in NetworkObjectId objectId, FieldId.Double fieldId, double value)
        {
            var key = new ObjectFieldId { fieldId = fieldId, objectId = objectId };
            fields[key] = value;
            return 0;
        }

        public byte Set(ushort clientId, in NetworkObjectId objectId, FieldId.Long fieldId, long value)
        {
            var key = new ObjectFieldId { fieldId = fieldId, objectId = objectId };
            fields[key] = value;
            return 0;
        }

        public byte Set(ushort clientId, in NetworkObjectId objectId, FieldId.Structure fieldId, ref NetworkBuffer value)
        {
            var key = new ObjectFieldId { fieldId = fieldId, objectId = objectId };
            fields[key] = value;
            return 0;
        }

        public byte Increment(ushort clientId, in NetworkObjectId objectId, FieldId.Double fieldId, double increment)
        {
            var key = new ObjectFieldId { fieldId = fieldId, objectId = objectId };
            fields[key] = (fields.GetValueOrDefault(key) as double?) + 1;
            return 0;
        }

        public byte Increment(ushort clientId, in NetworkObjectId objectId, FieldId.Long fieldId, long increment)
        {
            var key = new ObjectFieldId { fieldId = fieldId, objectId = objectId };
            fields[key] = (fields.GetValueOrDefault(key) as long?) + 1;
            return 0;
        }

        public byte DeleteField(ushort clientId, in NetworkObjectId objectId, FieldId fieldId)
        {
            var key = new ObjectFieldId { fieldId = fieldId, objectId = objectId };
            fields.Remove(key);
            return 0;
        }


        public byte CreateObject(ushort clientId, ushort template, ulong accessGroup, ref NetworkObjectId objectId)
        {
            objectId.memberId = memberId;
            objectId.id = idGenerator++;
            creatingObjects.Add(objectId, new NetworkObject(objectId, template));
            return 0;
        }

        public byte CreatedObject(ushort clientId, in NetworkObjectId objectId, bool roomOwner, ref NetworkBuffer singletonKey)
        {
            var createdObject = creatingObjects[objectId];
            creatingObjects.Remove(objectId);
            createdObjects.Add(objectId, createdObject);
            return 0;
        }

        public byte DeleteObject(ushort clientId, in NetworkObjectId objectId)
        {
            creatingObjects.Remove(objectId);
            createdObjects.Remove(objectId);
            return 0;
        }

        public byte Send(ushort clientId, in NetworkObjectId objectId, FieldId.Event fieldId, ref NetworkBuffer eventData)
        {
            var key = new ObjectFieldId { fieldId = fieldId, objectId = objectId };
            if (!events.TryGetValue(key, out var eventQueue))
            {
                eventQueue = new Queue<EventPayload>();
                events[key] = eventQueue;
            }

            eventQueue.Enqueue(new EventPayload { eventData = eventData });
            return 0;
        }

        public byte Send(ushort clientId, ushort targetUser, in NetworkObjectId objectId, FieldId.Event fieldId, ref NetworkBuffer eventData)
        {
            var key = new ObjectFieldId { fieldId = fieldId, objectId = objectId };
            if (!events.TryGetValue(key, out var eventQueue))
            {
                eventQueue = new Queue<EventPayload>();
                events[key] = eventQueue;
            }

            eventQueue.Enqueue(new EventPayload { eventData = eventData, targetUserOpt = targetUser });
            return 0;
        }

        public void ScheduleCommandFromServer(S2CCommands.CreateObject command)
        {
            s2cCommands.Add(new S2CCommand
            {
                commandType = CommandType.CreateGameObject,
                commandUnion = new S2CCommandUnion
                {
                    createObject = command
                }
            });

            creatingObjects.Add(command.objectId, new NetworkObject(command.objectId, command.template));
        }

        public void ScheduleCommandFromServer(S2CCommands.CreatedObject command)
        {
            s2cCommands.Add(new S2CCommand
            {
                commandType = CommandType.CreatedGameObject,
                commandUnion = new S2CCommandUnion
                {
                    createdObject = command
                }
            });
            var networkBuffer = new NetworkBuffer();
            CreatedObject(0, in command.objectId, false, ref networkBuffer);
        }

        public void ScheduleCommandFromServer(S2CCommands.SetLong command)
        {
            s2cCommands.Add(new S2CCommand
            {
                commandType = CommandType.SetLong,
                commandUnion = new S2CCommandUnion
                {
                    setLong = command
                }
            });
            Set(0, in command.objectId, new FieldId.Long(command.fieldId), command.value);
        }

        public void ScheduleCommandFromServer(S2CCommands.SetDouble command)
        {
            s2cCommands.Add(new S2CCommand
            {
                commandType = CommandType.SetDouble,
                commandUnion = new S2CCommandUnion
                {
                    setDouble = command
                }
            });
            Set(0, in command.objectId, new FieldId.Double(command.fieldId), command.value);
        }

        public void ScheduleCommandFromServer(S2CCommands.Event command)
        {
            s2cCommands.Add(new S2CCommand
            {
                commandType = CommandType.SendEvent,
                commandUnion = new S2CCommandUnion
                {
                    setEvent = command
                }
            });
            Send(0, in command.objectId, new FieldId.Event(command.fieldId), ref command.eventData);
        }

        public void ScheduleCommandFromServer(S2CCommands.SetStructure command)
        {
            s2cCommands.Add(new S2CCommand
            {
                commandType = CommandType.SetStructure,
                commandUnion = new S2CCommandUnion
                {
                    setStructure = command
                }
            });
            Set(0, in command.objectId, new FieldId.Structure(command.fieldId), ref command.value);
        }

        public void ScheduleCommandFromServer(S2CCommands.DeleteObject command)
        {
            s2cCommands.Add(new S2CCommand
            {
                commandType = CommandType.DeleteObject,
                commandUnion = new S2CCommandUnion
                {
                    deleteObject = command
                }
            });

            DeleteObject(0, in command.objectId);
        }


        public long? GetFieldValue(NetworkObjectId id, FieldId.Long field)
        {
            var key = new ObjectFieldId { fieldId = field, objectId = id };
            if (fields.TryGetValue(key, out var result))
            {
                return (long?)result;
            }

            return null;
        }

        public double? GetFieldValue(NetworkObjectId id, FieldId.Double field)
        {
            var key = new ObjectFieldId { fieldId = field, objectId = id };
            if (fields.TryGetValue(key, out var result))
            {
                return (double?)result;
            }

            return null;
        }

        public T? GetFieldValue<T>(NetworkObjectId id, FieldId.Structure field) where T : struct
        {
            var key = new ObjectFieldId { fieldId = field, objectId = id };
            if (fields.TryGetValue(key, out var result))
            {
                NetworkBuffer buffer = (NetworkBuffer)result;
                T item = new();
                codecRegistry.GetCodec<T>().Decode(ref buffer, ref item);
                return item;
            }

            return null;
        }

        public IEnumerable<T?> GetEvents<T>(NetworkObjectId id, FieldId.Event field) where T : struct
        {
            var key = new ObjectFieldId { fieldId = field, objectId = id };
            if (!events.TryGetValue(key, out var eventQueue)) yield break;

            while (eventQueue.Count > 0)
            {
                var ev = eventQueue.Dequeue();
                NetworkBuffer buffer = (NetworkBuffer)ev.eventData;
                T item = new();
                codecRegistry.GetCodec<T>().Decode(ref buffer, ref item);
                yield return item;
            }
        }

        public void ClearEvents()
        {
            events.Clear();
        }

        public long GetCreatedObjectsCount()
        {
            return createdObjects.Count;
        }

        public IList<NetworkObjectId> GetCreatedObjects()
        {
            return createdObjects.Keys.ToList();
        }

        public void SetMemberIdForNewNetworkObject(ushort memberId)
        {
            this.memberId = memberId;
        }

        public void Clear()
        {
            fields.Clear();
            events.Clear();
        }
    }
}
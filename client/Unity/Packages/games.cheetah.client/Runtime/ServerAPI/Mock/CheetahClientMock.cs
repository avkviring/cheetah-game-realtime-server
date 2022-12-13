using System.Collections.Generic;
using System.Linq;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.ServerAPI.Mock.Events;
using Games.Cheetah.Client.ServerAPI.Mock.Storage;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock
{
    public class CheetahClientMock : IServerAPI, ICheetahClientMock
    {
        internal readonly CodecRegistry codecRegistry;
        internal readonly ClientAPIMock client = new();
        internal readonly Structures structures = new();
        internal readonly Longs longs = new();
        internal readonly Doubles doubles = new();
        internal readonly Objects objects = new();
        internal readonly Storage.Events eventStorage = new();
        internal readonly Fields fields;

        public List<ICommandFromServer> commands = new();

        public CheetahClientMock(CodecRegistry codecRegistry)
        {
            this.codecRegistry = codecRegistry;
            fields = new Fields(longs, doubles, structures);
            client.OnReceive += OnReceive;
        }


        public void ScheduleCommandFromServer(ICommandFromServer command)
        {
            commands.Add(command);
        }


        public T? GetStructureValue<T>(CheetahObjectId objectId, ushort fieldId) where T : struct
        {
            var buffer = new CheetahBuffer();
            if (structures.TryGetFieldValue(objectId, fieldId, out buffer))
            {
                var result = new T();
                codecRegistry.GetCodec<T>().Decode(ref buffer, ref result);
                return result;
            }

            return null;
        }

        public long? GetLongValue(CheetahObjectId objectId, ushort fieldId)
        {
            if (longs.TryGetFieldValue(objectId, fieldId, out var value))
            {
                return value;
            }

            return null;
        }

        public double? GetDoubleValue(CheetahObjectId objectId, ushort fieldId)
        {
            if (doubles.TryGetFieldValue(objectId, fieldId, out var value))
            {
                return value;
            }

            return null;
        }

        public void SetMemberIdForNewCheetahObject(ushort memberId)
        {
            objects.memberId = memberId;
        }

        public long GetCreatedObjectsCount()
        {
            return objects.GetCreatedObjectsCount();
        }

        public void Clear()
        {
            structures.Clear();
            doubles.Clear();
            longs.Clear();
            objects.Clear();
        }


        public void OnReceive()
        {
            foreach (var serverEvent in commands)
            {
                serverEvent.Apply(this);
            }

            commands.Clear();
        }

        public IEnumerable<CheetahObjectId> GetCreatedObjects()
        {
            return objects.createdObjects.Select(item => item.objectId).ToList();
        }


        public IClientServerAPI Client => client;

        public IDoubleServerAPI Double => doubles;
        public IEventServerAPI Event => eventStorage;
        public IFieldServerAPI Field => fields;
        public ILongServerAPI Long => longs;
        public IObjectServerAPI Object => objects;
        public IStructureServerAPI Structure => structures;
    }
}
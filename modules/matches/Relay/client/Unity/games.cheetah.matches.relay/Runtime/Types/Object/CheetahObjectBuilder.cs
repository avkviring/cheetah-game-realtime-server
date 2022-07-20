using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Internal;
using Cheetah.Matches.Relay.Internal.FFI;

namespace Cheetah.Matches.Relay.Types.Object
{
    public struct CheetahObjectBuilder
    {
        private static CheetahBuffer buffer;
        private readonly CheetahObjectsCreateInfo createdInfo;
        private readonly ushort template;
        private readonly CheetahClient client;
        private CheetahObjectId objectId;

        internal CheetahObjectBuilder(
            ushort template,
            ulong accessGroup,
            CheetahObjectsCreateInfo createdInfo,
            CheetahClient client)
        {
            this.createdInfo = createdInfo;
            this.template = template;
            this.client = client;
            objectId = default;
            ResultChecker.Check(ObjectFFI.CreateObject(client.Id, template, accessGroup, ref objectId));
        }

        public CheetahObject Build()
        {
            buffer.Clear();
            ResultChecker.Check(ObjectFFI.Created(client.Id, ref objectId, false, ref buffer));
            createdInfo.OnLocalObjectCreating(ref objectId, template);
            createdInfo.OnLocalObjectCreate(ref objectId);
            return new CheetahObject(objectId, template, client);
        }

        /// <summary>
        /// Создать объект, принадлежащий комнате, такой объект не удаляется из комнаты при выходе игрока
        /// </summary>
        public void BuildRoomObject()
        {
            buffer.Clear();
            ResultChecker.Check(ObjectFFI.Created(client.Id, ref objectId, true, ref buffer));
        }

        /// <summary>
        /// Создать объект, принадлежащий комнате, в комнате может сущестовать только один объект с данным singletonKey,
        /// команды создания других объектов с таким же ключем будут игнорироваться сервером.
        /// </summary>
        public void BuildSingletonRoomObject<T>(ref T singletonKey) where T : struct
        {
            buffer.Clear();
            client.CodecRegistry.GetCodec<T>().Encode(ref singletonKey, ref buffer);
            ResultChecker.Check(ObjectFFI.Created(client.Id, ref objectId, true, ref buffer));
        }

        public CheetahObjectBuilder SetDouble(ushort fieldId, double value)
        {
            ResultChecker.Check(DoubleFFI.Set(client.Id, ref objectId, fieldId, value));
            return this;
        }

        public CheetahObjectBuilder SetStructure<T>(ushort fieldId, ref T item)
        {
            buffer.Clear();
            client.CodecRegistry.GetCodec<T>().Encode(ref item, ref buffer);
            ResultChecker.Check(StructureFFI.Set(client.Id, ref objectId, fieldId, ref buffer));
            return this;
        }

        public CheetahObjectBuilder SetLong(ushort fieldId, long value)
        {
            ResultChecker.Check(LongFFI.Set(client.Id, ref objectId, fieldId, value));
            return this;
        }

        public CheetahObjectBuilder CompareAndSetLong(ushort fieldId, long currentValue, long newValue)
        {
            ResultChecker.Check(LongFFI.CompareAndSet(client.Id, ref objectId, fieldId, currentValue, newValue, false, 0));
            return this;
        }

        public CheetahObjectBuilder CompareAndSetLongWithReset(ushort fieldId, long currentValue, long newValue, long resetValue)
        {
            ResultChecker.Check(LongFFI.CompareAndSet(client.Id, ref objectId, fieldId, currentValue, newValue, true, resetValue));
            return this;
        }

        public CheetahObjectBuilder CompareAndSetStructure<T>(ushort fieldId, ref T current, ref T newval)
        {
            buffer.Clear();
            var newBuffer = new CheetahBuffer();
            var resetBuffer = new CheetahBuffer();
            var codec = client.CodecRegistry.GetCodec<T>();
            codec.Encode(ref current, ref buffer);
            codec.Encode(ref newval, ref newBuffer);

            ResultChecker.Check(StructureFFI.CompareAndSet(
                client.Id,
                ref objectId,
                fieldId,
                ref buffer,
                ref newBuffer,
                false,
                ref resetBuffer
            ));

            return this;
        }

        public CheetahObjectBuilder CompareAndSetStructureWithReset<T>(ushort fieldId, ref T current, ref T newval, ref T reset)
        {
            buffer.Clear();
            var newBuffer = new CheetahBuffer();
            var resetBuffer = new CheetahBuffer();
            var codec = client.CodecRegistry.GetCodec<T>();
            codec.Encode(ref current, ref buffer);
            codec.Encode(ref newval, ref newBuffer);
            codec.Encode(ref reset, ref resetBuffer);

            ResultChecker.Check(StructureFFI.CompareAndSet(
                client.Id,
                ref objectId,
                fieldId,
                ref buffer,
                ref newBuffer,
                true,
                ref resetBuffer
            ));

            return this;
        }
    }
}
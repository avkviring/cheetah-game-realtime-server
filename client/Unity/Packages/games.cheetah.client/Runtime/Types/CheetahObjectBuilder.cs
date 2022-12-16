using Games.Cheetah.Client.Internal;

namespace Games.Cheetah.Client.Types
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
            ResultChecker.Check(client.serverAPI.Object.CreateObject(client.Id, template, accessGroup, ref objectId));
        }

        public CheetahObject Build()
        {
            buffer.Clear();
            ResultChecker.Check(client.serverAPI.Object.CreatedObject(client.Id, in objectId, false, ref buffer));
            createdInfo.OnLocalObjectCreating(in objectId, template);
            createdInfo.OnLocalObjectCreate(in objectId);
            return new CheetahObject(objectId, template);
        }

        /// <summary>
        /// Создать объект, принадлежащий комнате, такой объект не удаляется из комнаты при выходе игрока
        /// </summary>
        public void BuildRoomObject()
        {
            buffer.Clear();
            ResultChecker.Check(client.serverAPI.Object.CreatedObject(client.Id, in objectId, true, ref buffer));
        }

        /// <summary>
        /// Создать объект, принадлежащий комнате, в комнате может сущестовать только один объект с данным singletonKey,
        /// команды создания других объектов с таким же ключем будут игнорироваться сервером.
        /// </summary>
        public void BuildSingletonRoomObject<T>(ref T singletonKey) where T : struct
        {
            buffer.Clear();
            client.CodecRegistry.GetCodec<T>().Encode(in singletonKey, ref buffer);
            ResultChecker.Check(client.serverAPI.Object.CreatedObject(client.Id, in objectId, true, ref buffer));
        }

        public CheetahObjectBuilder SetDouble(FieldId.Double fieldId, double value)
        {
            ResultChecker.Check(client.serverAPI.Double.Set(client.Id, in objectId, fieldId, value));
            return this;
        }

        public CheetahObjectBuilder SetStructure<T>(FieldId.Structure fieldId, in T item)
        {
            buffer.Clear();
            client.CodecRegistry.GetCodec<T>().Encode(in item, ref buffer);
            ResultChecker.Check(client.serverAPI.Structure.Set(client.Id, in objectId, fieldId, ref buffer));
            return this;
        }

        public CheetahObjectBuilder SetLong(FieldId.Long fieldId, long value)
        {
            ResultChecker.Check(client.serverAPI.Long.Set(client.Id, in objectId, fieldId, value));
            return this;
        }

        public CheetahObjectBuilder CompareAndSetLong(FieldId.Long fieldId, long currentValue, long newValue, bool hasReset = false,
            long resetValue = default)
        {
            ResultChecker.Check(client.serverAPI.Long.CompareAndSet(client.Id, in objectId, fieldId, currentValue, newValue, hasReset, resetValue));
            return this;
        }


        public CheetahObjectBuilder CompareAndSetStructure<T>(FieldId.Structure fieldId, ref T current, ref T newval)
        {
            buffer.Clear();
            var newBuffer = new CheetahBuffer();
            var resetBuffer = new CheetahBuffer();
            var codec = client.CodecRegistry.GetCodec<T>();
            codec.Encode(in current, ref buffer);
            codec.Encode(in newval, ref newBuffer);

            ResultChecker.Check(client.serverAPI.Structure.CompareAndSet(
                client.Id,
                in objectId,
                fieldId,
                ref buffer,
                ref newBuffer,
                false,
                ref resetBuffer
            ));

            return this;
        }

        public CheetahObjectBuilder CompareAndSetStructure<T>(FieldId.Structure fieldId, ref T current, ref T newval, ref T reset)
        {
            buffer.Clear();
            var newBuffer = new CheetahBuffer();
            var resetBuffer = new CheetahBuffer();
            var codec = client.CodecRegistry.GetCodec<T>();
            codec.Encode(in current, ref buffer);
            codec.Encode(in newval, ref newBuffer);
            codec.Encode(in reset, ref resetBuffer);

            ResultChecker.Check(client.serverAPI.Structure.CompareAndSet(
                client.Id,
                in objectId,
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
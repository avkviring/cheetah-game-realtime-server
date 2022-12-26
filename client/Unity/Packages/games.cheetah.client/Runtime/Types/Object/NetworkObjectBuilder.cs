using Games.Cheetah.Client.Internal;
using Games.Cheetah.Client.Types.Field;

namespace Games.Cheetah.Client.Types.Object
{
    public struct NetworkObjectBuilder
    {
        private static NetworkBuffer buffer;
        private readonly ushort template;
        private readonly NetworkClient client;
        private NetworkObjectId objectId;

        internal NetworkObjectBuilder(
            ushort template,
            ulong accessGroup,
            NetworkClient client)
        {
            this.template = template;
            this.client = client;
            objectId = default;
            ResultChecker.Check(client.ffi.CreateObject(client.Id, template, accessGroup, ref objectId));
        }

        public NetworkObject Build()
        {
            buffer.Clear();

            ResultChecker.Check(client.ffi.CreatedObject(client.Id, in objectId, false, ref buffer));
            client.Reader.RegisterSelfObject(objectId, template);
            return new NetworkObject(objectId, template);
        }

        /// <summary>
        /// Создать объект, принадлежащий комнате, такой объект не удаляется из комнаты при выходе игрока
        /// </summary>
        public void BuildRoomObject()
        {
            buffer.Clear();
            ResultChecker.Check(client.ffi.CreatedObject(client.Id, in objectId, true, ref buffer));
        }

        /// <summary>
        /// Создать объект, принадлежащий комнате, в комнате может сущестовать только один объект с данным singletonKey,
        /// команды создания других объектов с таким же ключем будут игнорироваться сервером.
        /// </summary>
        public void BuildSingletonRoomObject<T>(ref T singletonKey) where T : struct
        {
            buffer.Clear();
            client.CodecRegistry.GetCodec<T>().Encode(in singletonKey, ref buffer);
            ResultChecker.Check(client.ffi.CreatedObject(client.Id, in objectId, true, ref buffer));
        }

        public NetworkObjectBuilder SetDouble(FieldId.Double fieldId, double value)
        {
            ResultChecker.Check(client.ffi.Set(client.Id, in objectId, fieldId, value));
            return this;
        }

        public NetworkObjectBuilder SetStructure<T>(FieldId.Structure fieldId, in T item)
        {
            buffer.Clear();
            client.CodecRegistry.GetCodec<T>().Encode(in item, ref buffer);
            ResultChecker.Check(client.ffi.Set(client.Id, in objectId, fieldId, ref buffer));
            return this;
        }

        public NetworkObjectBuilder SetLong(FieldId.Long fieldId, long value)
        {
            ResultChecker.Check(client.ffi.Set(client.Id, in objectId, fieldId, value));
            return this;
        }
    }
}
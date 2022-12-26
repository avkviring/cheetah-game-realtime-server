using System;
using System.Collections.Generic;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Types.Field;

namespace Games.Cheetah.Client.Types.Object
{
    /// <summary>
    /// Информация о созданном объекте
    /// </summary>
    public class NetworkObjectConstructor
    {
        public readonly NetworkObject NetworkObject;
        private readonly CodecRegistry codecRegistry;
        internal readonly Dictionary<ushort, NetworkBuffer> structures = new();
        internal readonly Dictionary<ushort, long> longs = new();
        internal readonly Dictionary<ushort, double> doubles = new();

        public NetworkObjectConstructor(NetworkObject networkObject, CodecRegistry codecRegistry)
        {
            NetworkObject = networkObject;
            this.codecRegistry = codecRegistry;
        }

        public bool TryGet<T>(FieldId.Structure fieldId, ref T item) where T : struct
        {
            if (structures.TryGetValue(fieldId.Id, out var buffer))
            {
                buffer.pos = 0;
                codecRegistry.GetCodec<T>().Decode(ref buffer, ref item);
                return true;
            }

            return false;
        }

        public void Get<T>(FieldId.Structure fieldId, ref T item) where T : struct
        {
            if (!TryGet(fieldId, ref item))
            {
                throw new CheetahObjectStructFieldNotFoundException(NetworkObject.ObjectId, fieldId.Id);
            }
        }


        public bool TryGet(FieldId.Long fieldId, out long value)
        {
            return longs.TryGetValue(fieldId.Id, out value);
        }

        public long Get(FieldId.Long fieldId)
        {
            if (TryGet(fieldId, out var value))
            {
                return value;
            }

            throw new CheetahObjectLongFieldNotFoundException(NetworkObject.ObjectId, fieldId.Id);
        }

        public bool TryGet(FieldId.Double fieldId, out double value)
        {
            return doubles.TryGetValue(fieldId.Id, out value);
        }

        public double Get(FieldId.Double fieldId)
        {
            if (TryGet(fieldId, out var value))
            {
                return value;
            }

            throw new CheetahObjectDoubleFieldNotFoundException(NetworkObject.ObjectId, fieldId.Id);
        }
    }

    public class CheetahObjectStructFieldNotFoundException : Exception
    {
        public CheetahObjectStructFieldNotFoundException(NetworkObjectId id, ushort fieldId) : base("Struct field " + fieldId +
            " not found in object with id " + id)
        {
        }
    }

    public class CheetahObjectLongFieldNotFoundException : Exception
    {
        public CheetahObjectLongFieldNotFoundException(NetworkObjectId id, ushort fieldId) : base("Long field " + fieldId +
                                                                                                  " not found in object with id " + id)
        {
        }
    }

    public class CheetahObjectDoubleFieldNotFoundException : Exception
    {
        public CheetahObjectDoubleFieldNotFoundException(NetworkObjectId id, ushort fieldId) : base("Double field " + fieldId +
            " not found in object with id " + id)
        {
        }
    }
}
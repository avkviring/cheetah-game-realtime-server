using System;
using System.Collections.Generic;
using Cheetah.Matches.Relay.Codec;

namespace Cheetah.Matches.Relay.Types.Object
{
    /// <summary>
    /// Информация о созданном объекте
    /// </summary>
    public class CheetahObjectConstructor
    {
        public readonly CheetahObject cheetahObject;
        private readonly CodecRegistry codecRegistry;
        internal readonly Dictionary<ushort, CheetahBuffer> structures = new Dictionary<ushort, CheetahBuffer>();
        internal readonly Dictionary<ushort, long> longs = new Dictionary<ushort, long>();
        internal readonly Dictionary<ushort, double> doubles = new Dictionary<ushort, double>();

        public CheetahObjectConstructor(CheetahObject cheetahObject, CodecRegistry codecRegistry)
        {
            this.cheetahObject = cheetahObject;
            this.codecRegistry = codecRegistry;
        }

        public bool TryGetStruct<T>(ushort fieldId, ref T item) where T : struct
        {
            if (structures.TryGetValue(fieldId, out var buffer))
            {
                buffer.pos = 0;
                codecRegistry.GetCodec<T>().Decode(ref buffer, ref item);
                return true;
            }

            return false;
        }

        public void GetStruct<T>(ushort fieldId, ref T item) where T : struct
        {
            if (!TryGetStruct(fieldId, ref item))
            {
                throw new CheetahObjectStructFieldNotFoundException(cheetahObject.ObjectId, fieldId);
            }
        }


        public bool TryGetLong(ushort fieldId, out long value)
        {
            return longs.TryGetValue(fieldId, out value);
        }

        public long GetLong(ushort fieldId)
        {
            if (TryGetLong(fieldId, out var value))
            {
                return value;
            }

            throw new CheetahObjectLongFieldNotFoundException(cheetahObject.ObjectId, fieldId);
        }

        public bool TryGetDouble(ushort fieldId, out double value)
        {
            return doubles.TryGetValue(fieldId, out value);
        }

        public double GetDouble(ushort fieldId)
        {
            if (TryGetDouble(fieldId, out var value))
            {
                return value;
            }

            throw new CheetahObjectDoubleFieldNotFoundException(cheetahObject.ObjectId, fieldId);
        }
    }

    public class CheetahObjectStructFieldNotFoundException : Exception
    {
        public CheetahObjectStructFieldNotFoundException(CheetahObjectId id, ushort fieldId) : base("Struct field " + fieldId +
            " not found in object with id " + id)
        {
        }
    }

    public class CheetahObjectLongFieldNotFoundException : Exception
    {
        public CheetahObjectLongFieldNotFoundException(CheetahObjectId id, ushort fieldId) : base("Long field " + fieldId +
                                                                                                  " not found in object with id " + id)
        {
        }
    }

    public class CheetahObjectDoubleFieldNotFoundException : Exception
    {
        public CheetahObjectDoubleFieldNotFoundException(CheetahObjectId id, ushort fieldId) : base("Double field " + fieldId +
            " not found in object with id " + id)
        {
        }
    }
}
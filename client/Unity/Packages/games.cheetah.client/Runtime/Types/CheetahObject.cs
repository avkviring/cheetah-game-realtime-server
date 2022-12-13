using System;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client
{
    /// <summary>
    /// Сетевой игровой объект
    /// </summary>
    public struct CheetahObject : IEquatable<CheetahObject>
    {
        public CheetahObjectId ObjectId;
        public ushort Template;

        public CheetahObject(CheetahObjectId objectId, ushort template)
        {
            ObjectId = objectId;
            Template = template;
        }


        public override string ToString()
        {
            return $"{nameof(ObjectId)}: {ObjectId}, {nameof(Template)}: {Template}";
        }

        public bool Equals(CheetahObject other)
        {
            return ObjectId.Equals(other.ObjectId);
        }

        public override bool Equals(object obj)
        {
            return obj is CheetahObject other && Equals(other);
        }

        public override int GetHashCode()
        {
            return ObjectId.GetHashCode();
        }
    }
}
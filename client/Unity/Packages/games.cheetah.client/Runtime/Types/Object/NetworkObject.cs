using System;

namespace Games.Cheetah.Client.Types.Object
{
    /// <summary>
    /// Сетевой игровой объект
    /// </summary>
    public struct NetworkObject : IEquatable<NetworkObject>
    {
        public NetworkObjectId ObjectId;
        public ushort Template;

        public NetworkObject(NetworkObjectId objectId, ushort template)
        {
            ObjectId = objectId;
            Template = template;
        }


        public override string ToString()
        {
            return $"{nameof(ObjectId)}: {ObjectId}, {nameof(Template)}: {Template}";
        }

        public bool Equals(NetworkObject other)
        {
            return ObjectId.Equals(other.ObjectId);
        }

        public override bool Equals(object obj)
        {
            return obj is NetworkObject other && Equals(other);
        }

        public override int GetHashCode()
        {
            return ObjectId.GetHashCode();
        }
    }
}
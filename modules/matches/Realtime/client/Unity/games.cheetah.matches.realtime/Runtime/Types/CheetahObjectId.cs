using System.Runtime.InteropServices;

namespace Cheetah.Matches.Realtime.Types
{
    [StructLayout(LayoutKind.Sequential)]
    public struct CheetahObjectId
    {
        public uint id;

        /// <summary>
        /// Владельцем объекта может быть или комната или пользователь
        /// </summary>
        [MarshalAs(UnmanagedType.I1)] public bool roomOwner;

        /// <summary>
        /// Идентификатор участника комнаты
        /// </summary>
        public ushort memberId;

        public static CheetahObjectId Empty;

        public override string ToString()
        {
            return $"{nameof(id)}: {id}, {nameof(roomOwner)}: {roomOwner}, {nameof(memberId)}: {memberId}";
        }

        public static bool operator ==(CheetahObjectId a, CheetahObjectId b)
            => a.Equals(b);

        public static bool operator !=(CheetahObjectId a, CheetahObjectId b)
            => !(a == b);

        public override bool Equals(object obj)
        {
            return obj is CheetahObjectId other && Equals(other);
        }

        public bool Equals(CheetahObjectId other)
        {
            return id == other.id && roomOwner == other.roomOwner && memberId == other.memberId;
        }

        public override int GetHashCode()
        {
            unchecked
            {
                var hashCode = (int)id;
                hashCode = (hashCode * 397) ^ roomOwner.GetHashCode();
                hashCode = (hashCode * 397) ^ memberId;
                return hashCode;
            }
        }
    }
}
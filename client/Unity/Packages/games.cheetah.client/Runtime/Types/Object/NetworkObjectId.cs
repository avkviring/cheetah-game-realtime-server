using System;
using System.Runtime.InteropServices;

namespace Games.Cheetah.Client.Types.Object
{
    [StructLayout(LayoutKind.Sequential)]
    public struct NetworkObjectId : IEquatable<NetworkObjectId>
    {
        // pub id: u32,
        // is_room_owner: bool,
        // member_id: RoomMemberId,
        //     
        public uint id;

        /// <summary>
        /// Владельцем объекта может быть или комната или пользователь
        /// </summary>
        [MarshalAs(UnmanagedType.I1)] public bool IsRoomOwner;

        /// <summary>
        /// Идентификатор участника комнаты
        /// </summary>
        public ulong memberId;

        public static NetworkObjectId Empty;

        public override string ToString()
        {
            return $"{nameof(id)}: {id}, {nameof(IsRoomOwner)}: {IsRoomOwner}, {nameof(memberId)}: {memberId}";
        }

        public static bool operator ==(NetworkObjectId a, NetworkObjectId b)
            => a.Equals(b);

        public static bool operator !=(NetworkObjectId a, NetworkObjectId b)
            => !(a == b);

        public override bool Equals(object obj)
        {
            return obj is NetworkObjectId other && Equals(other);
        }

        public bool Equals(NetworkObjectId other)
        {
            return id == other.id && IsRoomOwner == other.IsRoomOwner && memberId == other.memberId;
        }

        public override int GetHashCode()
        {
            unchecked
            {
                var hashCode = (int)id;
                hashCode = (hashCode * 397) ^ IsRoomOwner.GetHashCode();
                hashCode = (hashCode * 397) ^ memberId.GetHashCode();
                return hashCode;
            }
        }
    }
}
#nullable enable
using System;
using Cheetah.Matches.Realtime.EmbeddedServer.API;
using Cheetah.Matches.Realtime.EmbeddedServer.FFI;

namespace Cheetah.Matches.Realtime.EmbeddedServer.Impl
{
    internal class ServerRoomImpl : ServerRoom
    {
        private readonly ulong roomId;
        private readonly Server.Description serverDescription;
        

        internal ServerRoomImpl(Server.Description serverDescription, ulong roomId)
        {
            this.serverDescription = serverDescription;
            this.roomId = roomId;
        }

        public ServerMember CreateMember(ulong group)
        {
            var member = new Member.MemberDescription();
            if (!Member.CreateMember(serverDescription.id, roomId, group, ref member, OnMemberError))
            {
                throw new Exception("Cannot create member. " + onMemberErrorMessage);
            }

            return new ServerMemberImpl(member);
        }


#if UNITY
        [MonoPInvokeCallback(typeof(Member.OnMemberError))]
#endif
        private static void OnMemberError(string? message)
        {
            onMemberErrorMessage = message;
        }

        private static string? onMemberErrorMessage;

        public ulong GetId()
        {
            return roomId;
        }

        public override string ToString()
        {
            return $"{nameof(serverDescription)}: {serverDescription}, {nameof(roomId)}: {roomId}";
        }
    }
}
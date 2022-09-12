using Cheetah.Matches.Realtime.EmbeddedServer.API;
using Cheetah.Matches.Realtime.EmbeddedServer.FFI;

namespace Cheetah.Matches.Realtime.EmbeddedServer.Impl
{
    internal class ServerMemberImpl : ServerMember
    {
        private readonly Member.MemberDescription member;

        internal ServerMemberImpl(Member.MemberDescription member)
        {
            this.member = member;
        }

        public uint GetId()
        {
            return member.id;
        }

        public byte[] GetPrivateKey()
        {
            unsafe
            {
                var result = new byte[32];
                for (int i = 0; i < 32; i++)
                {
                    result[i] = member.privateKey[i];
                }

                return result;
            }
        }
    }
}
using System.Runtime.InteropServices;

namespace Cheetah.Matches.Realtime.EmbeddedServer.FFI
{
    internal static class Member
    {
        internal const int PrivateKeyLength = 32;

        internal struct MemberDescription
        {
            internal ushort id;

            [MarshalAs(UnmanagedType.ByValArray, SizeConst = PrivateKeyLength)]
            internal unsafe fixed byte privateKey[PrivateKeyLength];
        }


        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "create_member")]
        internal static extern bool CreateMember(ulong serverId, ulong roomId, ulong group, ref MemberDescription description);
    }
}
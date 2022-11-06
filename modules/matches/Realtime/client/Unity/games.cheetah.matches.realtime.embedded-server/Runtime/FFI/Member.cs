using System.Runtime.InteropServices;

namespace Cheetah.Matches.Realtime.EmbeddedServer.FFI
{
    internal static class Member
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void OnMemberError([MarshalAs(UnmanagedType.LPWStr)] string message);


        internal const int PrivateKeyLength = 32;

        internal struct MemberDescription
        {
            internal ushort id;

            internal unsafe fixed byte privateKey[PrivateKeyLength];
        }


        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "create_member")]
        internal static extern bool CreateMember(ulong serverId, ulong roomId, ulong group, ref MemberDescription description,
            OnMemberError onMemberError);
    }
}
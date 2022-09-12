using System.Runtime.InteropServices;

namespace Cheetah.Matches.Realtime.EmbeddedServer.FFI
{
    internal static class Server
    {
        internal struct Description
        {
            internal ulong id;

            [MarshalAs(UnmanagedType.ByValArray, SizeConst = 4)]
            internal unsafe fixed byte serverIp[4];

            internal ushort gamePort;
        }


        public enum ResultCode
        {
            Ok = 0,
            InternalError = 1,
            BindingAddressNotIpV4 = 2,
        }

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "run_new_server")]
        internal static extern ResultCode RunNewServer(ref Description description);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "destroy_server")]
        internal static extern bool DestroyServer(ulong serverId);
    }
}
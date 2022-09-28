using System.Runtime.InteropServices;

namespace Cheetah.Matches.Realtime.EmbeddedServer.FFI
{
    internal static class Server
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void OnServerError([MarshalAs(UnmanagedType.LPWStr)] string message);

        internal struct Description
        {
            internal ulong id;

            [MarshalAs(UnmanagedType.ByValArray, SizeConst = 4)]
            internal unsafe fixed byte serverIp[4];

            internal ushort gamePort;
        }

        
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "run_new_server")]
        internal static extern bool RunNewServer(ref Description description, OnServerError onServerError);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "destroy_server")]
        internal static extern bool DestroyServer(ulong serverId);
    }
}
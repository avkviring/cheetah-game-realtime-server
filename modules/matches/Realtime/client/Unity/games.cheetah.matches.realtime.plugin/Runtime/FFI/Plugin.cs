using System.Runtime.InteropServices;

namespace Cheetah.Matches.Realtime.Plugin.FFI
{
    internal static class Plugin
    {
        [StructLayout(LayoutKind.Sequential)]
        internal struct RoomEvent
        {
            [MarshalAs(UnmanagedType.U8)] public  ulong roomId;
            public RoomEventType eventType;
        }

        internal enum RoomEventType
        {
            Created = 0,
            Deleted = 1,
        }

        public enum ResultCode
        {
            OK = 0,
            Empty = 1,
            Error = 2,
        }

        [StructLayout(LayoutKind.Sequential)]
        public unsafe struct NativeString
        {
            public byte size;

            [MarshalAs(UnmanagedType.ByValArray, SizeConst = 256)]
            public fixed sbyte values[256];
        }


        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "create_plugin")]
        internal static extern ResultCode CreatePlugin([MarshalAs(UnmanagedType.LPStr)] string grpcServerAddr, out ushort serverPluginId);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "pop_room_event")]
        internal static extern ResultCode PopRoomEvent(ushort pluginId, out RoomEvent roomEvent);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "get_last_error_msg")]
        public static extern void GetLastErrorMsg(out NativeString buffer);
    }
}
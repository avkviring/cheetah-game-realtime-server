using System.Runtime.InteropServices;

namespace Cheetah.Matches.Realtime.EmbeddedServer.FFI
{
    internal static class Room
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void OnRoomError([MarshalAs(UnmanagedType.LPWStr)] string message);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "create_room")]
        internal static extern bool CreateRoom(ulong serverId, ref ulong roomId, OnRoomError onRoomError);
    }
}
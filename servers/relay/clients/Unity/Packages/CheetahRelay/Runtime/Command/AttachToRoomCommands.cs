using System.Runtime.InteropServices;

namespace CheetahRelay
{
    public static class AttachToRoomCommand
    {
        /// <summary>
        /// Присоединиться к комнате
        /// </summary>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "attach_to_room")]
        public static extern bool AttachToRoom();
    }
}
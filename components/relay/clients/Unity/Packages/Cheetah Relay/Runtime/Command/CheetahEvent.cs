using System.Runtime.InteropServices;

namespace CheetahRelay
{
    public static class CheetahEvent
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Listener(ref CheetahCommandMeta meta, ref CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data);

        /// <summary>
        /// Установить обработчик серверных команд для текущего клиента
        /// </summary>
        /// <param name="listener"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_event_listener")]
        public static extern bool SetListener(Listener listener);


        /// <summary>
        /// Отправить событие всем клиента данного игрового объекта
        /// </summary>
        /// <param name="objectId"></param>
        /// <param name="fieldId"></param>
        /// <param name="data"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "send_event")]
        public static extern bool Send(ref CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data);

        /// <summary>
        /// Отправить событие всем клиента данного игрового объекта
        /// </summary>
        /// <param name="targetUser">Сообщение будет доставлено только данному пользователю</param>
        /// <param name="objectId"></param>
        /// <param name="fieldId"></param>
        /// <param name="data"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "send_target_event")]
        public static extern bool Send(ushort targetUser, ref CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data);
    }
}
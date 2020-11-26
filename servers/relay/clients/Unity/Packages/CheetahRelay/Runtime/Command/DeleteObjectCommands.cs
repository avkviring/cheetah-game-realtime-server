using System.Runtime.InteropServices;

namespace CheetahRelay
{
    public static class DeleteObjectCommand
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Listener(ref CommandMeta meta, ref RelayObjectId objectId);

        /// <summary>
        /// Установить обработчик серверных команд для текущего клиента
        /// </summary>
        /// <param name="objectListener"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_delete_object_listener")]
        public static extern bool SetListener(Listener objectListener);


        /// <summary>
        /// Удалить объект
        /// </summary>
        /// <param name="objectId"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "delete_object")]
        public static extern bool Delete(ref RelayObjectId objectId);
    }
}
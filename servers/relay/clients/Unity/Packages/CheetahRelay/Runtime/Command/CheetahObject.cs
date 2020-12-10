using System.Runtime.InteropServices;
using System.Text;

namespace CheetahRelay
{
    public static class CheetahObject
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void CreateListener(ref CheetahCommandMeta meta, ref CheetahObjectId objectId, ushort template);
        
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void CreatedListener(ref CheetahCommandMeta meta, ref CheetahObjectId objectId);


        /// <summary>
        /// Установить обработчик серверных команд для текущего клиента
        /// </summary>
        /// <param name="listener"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_create_object_listener")]
        public static extern bool SetCreateListener(CreateListener listener);


        /// <summary>
        /// Установить обработчик серверных команд для текущего клиента
        /// </summary>
        /// <param name="listener"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_created_object_listener")]
        public static extern bool SetCreatedListener(CreatedListener listener);

        /// <summary>
        /// Создать объект
        /// </summary>
        /// <param name="template"></param>
        /// <param name="accessGroup"></param>
        /// <param name="objectId"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "create_object")]
        public static extern bool Create(ushort template, ulong accessGroup, ref CheetahObjectId objectId);

        /// <summary>
        /// Объект создан
        /// </summary>
        /// <param name="objectId"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "created_object")]
        public static extern bool Created(ref CheetahObjectId objectId);


        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void DeleteListener(ref CheetahCommandMeta meta, ref CheetahObjectId objectId);

        /// <summary>
        /// Установить обработчик серверных команд для текущего клиента
        /// </summary>
        /// <param name="objectDeleteListener"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_delete_object_listener")]
        public static extern bool SetDeleteListener(DeleteListener objectDeleteListener);


        /// <summary>
        /// Удалить объект
        /// </summary>
        /// <param name="objectId"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "delete_object")]
        public static extern bool Delete(ref CheetahObjectId objectId);
    }
}
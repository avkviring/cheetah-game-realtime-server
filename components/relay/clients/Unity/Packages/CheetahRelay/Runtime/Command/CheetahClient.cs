using System.Runtime.InteropServices;

namespace CheetahRelay
{
    public static class CheetahClient
    {
        /// <summary>
        /// Создать клиента и установить соединение с сервером.
        /// Вновь созданный клиент устанавливается в качестве клиента по-умолчанию.
        /// </summary>
        /// <param name="serverAddress"></param>
        /// <param name="userId"></param>
        /// <param name="userPrivateKey"></param>
        /// <param name="startFrameId">Начальный идентификатор фрейма, 0 - при первом входе в комнату, N - при повторном входе в ту же самую комнату</param>
        /// <param name="clientId">Результат - локальный идентификатор клиента</param>
        /// <returns>false - ошибка создания сетевего сокета</returns>
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "create_client")]
        public static extern bool CreateClient(
            [MarshalAs(UnmanagedType.LPStr)] string serverAddress,
            uint userId,
            ref CheetahBuffer userPrivateKey,
            ulong startFrameId,
            out ushort clientId
        );
        

        /// <summary>
        /// Установить текущего клиента.
        /// Используется в основном для тестов, так как CreateClient также вызывает данный метод
        /// </summary>
        /// <param name="client"></param>
        /// <returns>true - успешно, false - клиента с таким идентификатором не найден</returns>
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_current_client")]
        public static extern bool SetCurrentClient(ushort client);


        /// <summary>
        ///  Получить статус сетевого соединения с сервером
        /// </summary>
        /// <param name="status"> результат</param>
        /// <returns>true - успешно, false - клиента с таким идентификатором не найден</returns>
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "get_connection_status")]
        public static extern bool GetConnectionStatus(out CheetahConnectionStatus status);

        /// <summary>
        ///  Получить id текущего фрейма
        /// </summary>
        /// <param name="frameId"></param>
        /// <returns>true - успешно, false - клиента с таким идентификатором не найден</returns>
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "get_frame_id")]
        public static extern bool GetFrameId(out ulong frameId);


        /// <summary>
        /// Обработать входящие команды
        /// </summary>
        /// <returns>false - клиент не найден</returns>
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "receive")]
        public static extern bool Receive();


        /// <summary>
        /// Разорвать соединение и удалить клиента
        /// </summary>
        /// <returns>false - клиент не найден</returns>
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "destroy_client")]
        public static extern bool DestroyClient();
        
        
        /// <summary>
        /// Присоединиться к комнате
        /// </summary>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "attach_to_room")]
        public static extern bool AttachToRoom();
        
         /// <summary>
        /// Установить канал отправки все последующих команд
        /// </summary>
        /// <param name="channelType">тип канала</param>
        /// <param name="group">группа, для групповых каналов, для остальных игнорируется</param>
        /// <returns>false - клиент не найден</returns>
        /// TODO - подумать над group, так как он применим не для всех каналов
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_channel")]
        public static extern bool SetChannelType(ChannelType channelType, ushort group);
        

        public enum ChannelType
        {
            /// <summary>
            /// - гарантия доставки
            /// - нет гарантии порядка выполнения
            /// </summary>
            ReliableUnordered,

            /// <summary>
            /// - нет гарантии доставки
            /// - нет гарантии порядка выполнения
            /// </summary>
            UnreliableUnordered,

            /// <summary>
            /// - гарантия доставки
            /// - не выполняются предыдущие команды в рамках объекта 
            /// </summary>
            ReliableOrderedByObject,

            /// <summary>
            /// - нет гарантии доставки
            /// - не выполняются предыдущие команды в рамках объекта
            /// </summary>
            UnreliableOrderedByObject,

            /// <summary>
            /// - гарантия доставки
            /// - команды выполняются строго последовательно в рамках объекта
            /// </summary>
            ReliableSequenceByObject,

            /// <summary>
            /// - гарантия доставки
            /// - не выполняются предыдущие команды в рамках группы
            /// </summary>
            ReliableOrderedByGroup,


            /// <summary>
            /// - нет гарантии доставки
            /// - не выполняются предыдущие команды в рамках объекта
            /// </summary>
            UnreliableOrderedByGroup,

            /// <summary>
            /// - гарантия доставки
            /// - команды выполняются строго последовательно в рамках группы
            /// </summary>
            ReliableSequenceByGroup,
        }
    }
}
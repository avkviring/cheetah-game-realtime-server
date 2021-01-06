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
        /// <param name="roomId"></param>
        /// <param name="userPrivateKey"></param>
        /// <param name="startFrameId">Начальный идентификатор фрейма, 0 - при первом входе в комнату, N - при повторном входе в ту же самую комнату</param>
        /// <param name="clientId">Результат - локальный идентификатор клиента</param>
        /// <returns>false - ошибка создания сетевого сокета</returns>
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "create_client")]
        public static extern bool CreateClient(
            [MarshalAs(UnmanagedType.LPStr)] string serverAddress,
            ushort userId,
            ulong roomId,
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
        /// Установить объект - источник команды в мета информацию
        /// </summary>
        /// <param name="objectId"></param>
        /// <returns></returns>
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_source_object_to_meta")]
        public static extern bool SetSourceObjectToMeta(ref CheetahObjectId objectId);


        /// <summary>
        ///  Получить статус сетевого соединения с сервером
        /// </summary>
        /// <param name="status"> результат</param>
        /// <returns>true - успешно, false - клиента с таким идентификатором не найден</returns>
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "get_connection_status")]
        public static extern bool GetConnectionStatus(out CheetahConnectionStatus status);

        /// <summary>
        ///  Получить статистику текущего клиента
        /// </summary>
        /// <returns>true - успешно, false - клиента с таким идентификатором не найден</returns>
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "get_statistics")]
        public static extern bool GetStatistics(out CheetahStatistics statistics);


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
        /// Отсоединиться от комнаты
        /// </summary>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "detach_from_room")]
        public static extern bool DetachFromRoom();

        /// <summary>
        /// Установить канал отправки все последующих команд
        /// </summary>
        /// <param name="channelType">тип канала</param>
        /// <param name="group">группа, для групповых каналов, для остальных игнорируется</param>
        /// <returns>false - клиент не найден</returns>
        /// TODO - подумать над group, так как он применим не для всех каналов
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_channel")]
        public static extern bool SetChannelType(ChannelType channelType, ushort group);

        
        /// <summary>
        /// Задать параметры эмуляции RTT
        /// Подробнее смотрите в документации проекта
        /// </summary>
        /// <param name="rttInMs"></param>
        /// <param name="rttDispersion"></param>
        /// <returns></returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_rtt_emulation")]
        public static extern bool SetRttEmulation(ulong rttInMs, double rttDispersion);


        /// <summary>
        /// Задать параметры эмуляции потери пакетов
        /// Подробнее смотрите в документации проекта
        /// </summary>
        /// <param name="dropProbability"></param>
        /// <param name="dropTimeInMs"></param>
        /// <returns></returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_drop_emulation")]
        public static extern bool SetDropEmulation(double dropProbability, ulong dropTimeInMs);

        /// <summary>
        /// Сброс эмуляции параметров сети
        /// </summary>
        /// <returns></returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "reset_emulation")]
        public static extern bool ResetEmulation();
        
        

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
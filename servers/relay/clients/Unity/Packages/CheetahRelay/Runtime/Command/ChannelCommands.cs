using System.Runtime.InteropServices;

namespace CheetahRelay
{
    public static class ChannelCommands
    {
        /// <summary>
        /// Установить канал отправки все последующих команд
        /// </summary>
        /// <param name="channel">тип канала</param>
        /// <param name="group">группа, для групповых каналов, для остальных игнорируется</param>
        /// <returns>false - клиент не найден</returns>
        /// TODO - подумать над group, так как он применим не для всех каналов
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_channel")]
        public static extern bool SetChannel(Channel channel, ushort group);
    }

    public enum Channel
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
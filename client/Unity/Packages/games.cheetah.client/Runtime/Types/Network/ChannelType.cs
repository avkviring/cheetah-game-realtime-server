namespace Games.Cheetah.Client.Types.Network
{
    /// <summary>
    /// Тип канала для отправки команд
    /// </summary>
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
        /// - не выполняются предыдущие команды в рамках группы
        /// </summary>
        ReliableOrdered,
        
        /// <summary>
        /// - нет гарантии доставки
        /// - не выполняются предыдущие команды в рамках объекта
        /// </summary>
        UnreliableOrdered,

        /// <summary>
        /// - гарантия доставки
        /// - команды выполняются строго последовательно в рамках группы
        /// </summary>
        ReliableSequence
    }
}
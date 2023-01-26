using System;

namespace Games.Cheetah.Client.Types.Network
{
    /// <summary>
    /// Тип канала для отправки команд
    /// </summary>
    public enum NetworkChannelType
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


    public class NetworkChannel : IEquatable<NetworkChannel>
    {
        /// <summary>
        /// - гарантия доставки
        /// - команды выполняются строго последовательно в рамках группы
        /// </summary>
        public static readonly NetworkChannel ReliableSequence = new(NetworkChannelType.ReliableSequence, 1);

        /// <summary>
        /// - гарантия доставки
        /// - не выполняются предыдущие команды в рамках группы
        /// </summary>
        public static readonly NetworkChannel ReliableOrdered = new(NetworkChannelType.ReliableOrdered, 1);

        /// <summary>
        /// - гарантия доставки
        /// - нет гарантии порядка выполнения
        /// </summary>
        public static readonly NetworkChannel ReliableUnordered = new(NetworkChannelType.ReliableUnordered);


        /// <summary>
        /// - нет гарантии доставки
        /// - нет гарантии порядка выполнения
        /// </summary>
        public static readonly NetworkChannel UnreliableUnordered = new(NetworkChannelType.UnreliableUnordered);

        /// <summary>
        /// - нет гарантии доставки
        /// - не выполняются предыдущие команды в рамках объекта
        /// </summary>
        public static readonly NetworkChannel UnreliableOrdered = new(NetworkChannelType.UnreliableOrdered);

        public static readonly NetworkChannel Default = ReliableSequence;

        public readonly NetworkChannelType NetworkChannelType;
        public readonly byte group;

        public NetworkChannel(NetworkChannelType networkChannelType, byte group = 0)
        {
            this.NetworkChannelType = networkChannelType;
            this.group = group;
        }

        public bool Equals(NetworkChannel other)
        {
            if (ReferenceEquals(null, other)) return false;
            if (ReferenceEquals(this, other)) return true;
            return NetworkChannelType == other.NetworkChannelType && group == other.group;
        }

        public override bool Equals(object obj)
        {
            if (ReferenceEquals(null, obj)) return false;
            if (ReferenceEquals(this, obj)) return true;
            if (obj.GetType() != this.GetType()) return false;
            return Equals((NetworkChannel)obj);
        }

        public override int GetHashCode()
        {
            unchecked
            {
                return ((int)NetworkChannelType * 397) ^ group.GetHashCode();
            }
        }
    }
}
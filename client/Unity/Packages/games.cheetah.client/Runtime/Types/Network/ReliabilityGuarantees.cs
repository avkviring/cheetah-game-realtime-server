using System;

namespace Games.Cheetah.Client.Types.Network
{
    /// <summary>
    /// Гарантии доставки
    /// </summary>
    public enum ReliabilityGuarantees
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


    public class ReliabilityGuaranteesChannel : IEquatable<ReliabilityGuaranteesChannel>
    {
        /// <summary>
        /// - гарантия доставки
        /// - команды выполняются строго последовательно в рамках группы
        /// </summary>
        public static readonly ReliabilityGuaranteesChannel ReliableSequence = new(ReliabilityGuarantees.ReliableSequence, 1);

        /// <summary>
        /// - гарантия доставки
        /// - не выполняются предыдущие команды в рамках группы
        /// </summary>
        public static readonly ReliabilityGuaranteesChannel ReliableOrdered = new(ReliabilityGuarantees.ReliableOrdered, 1);

        /// <summary>
        /// - гарантия доставки
        /// - нет гарантии порядка выполнения
        /// </summary>
        public static readonly ReliabilityGuaranteesChannel ReliableUnordered = new(ReliabilityGuarantees.ReliableUnordered);


        /// <summary>
        /// - нет гарантии доставки
        /// - нет гарантии порядка выполнения
        /// </summary>
        public static readonly ReliabilityGuaranteesChannel UnreliableUnordered = new(ReliabilityGuarantees.UnreliableUnordered);

        /// <summary>
        /// - нет гарантии доставки
        /// - не выполняются предыдущие команды в рамках объекта
        /// </summary>
        public static readonly ReliabilityGuaranteesChannel UnreliableOrdered = new(ReliabilityGuarantees.UnreliableOrdered);

        public static readonly ReliabilityGuaranteesChannel Default = ReliableSequence;

        public readonly ReliabilityGuarantees ReliabilityGuarantees;
        public readonly byte group;

        public ReliabilityGuaranteesChannel(ReliabilityGuarantees reliabilityGuarantees, byte group = 0)
        {
            this.ReliabilityGuarantees = reliabilityGuarantees;
            this.group = group;
        }

        public bool Equals(ReliabilityGuaranteesChannel other)
        {
            if (ReferenceEquals(null, other)) return false;
            if (ReferenceEquals(this, other)) return true;
            return ReliabilityGuarantees == other.ReliabilityGuarantees && group == other.group;
        }

        public override bool Equals(object obj)
        {
            if (ReferenceEquals(null, obj)) return false;
            if (ReferenceEquals(this, obj)) return true;
            if (obj.GetType() != this.GetType()) return false;
            return Equals((ReliabilityGuaranteesChannel)obj);
        }

        public override int GetHashCode()
        {
            unchecked
            {
                return ((int)ReliabilityGuarantees * 397) ^ group.GetHashCode();
            }
        }
    }
}
using System.Runtime.InteropServices;

namespace Cheetah.Matches.Relay.Types
{
    [StructLayout(LayoutKind.Sequential)]
    public struct CheetahClientStatistics
    {
        /// <summary>
        /// Идентификатор последнего отправленного фрейма
        /// </summary>
        public ulong LastFrameId;

        /// <summary>
        /// Время прохождения пакета от клиента к серверу и обратно
        /// </summary>
        public ulong RttInMs;

        /// <summary>
        /// Среднее скользящее количество переотправленных фреймов за 5 секунд
        /// </summary>
        public uint AverageRetransmitFrames;

        /// <summary>
        /// Количество принятых пакетов
        /// </summary>
        public ulong ReceivePacketCount;

        /// <summary>
        /// Количество отправленных пакетов
        /// </summary>
        public ulong SendPacketCount;

        /// <summary>
        /// Размер всех принятых данных
        /// </summary>
        public ulong ReceiveSize;

        /// <summary>
        /// Размер всех отправленных данных
        /// </summary>
        public ulong SendSize;

        public override string ToString()
        {
            return
                $"{nameof(LastFrameId)}: {LastFrameId}, {nameof(RttInMs)}: {RttInMs}, {nameof(AverageRetransmitFrames)}: {AverageRetransmitFrames}, {nameof(ReceivePacketCount)}: {ReceivePacketCount}, {nameof(SendPacketCount)}: {SendPacketCount}, {nameof(ReceiveSize)}: {ReceiveSize}, {nameof(SendSize)}: {SendSize}";
        }
    }
}
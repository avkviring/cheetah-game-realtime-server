using System.Runtime.InteropServices;

namespace Games.Cheetah.Client.Types.Network
{
    [StructLayout(LayoutKind.Sequential)]
    public struct Statistics
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
                $"{nameof(LastFrameId)}: {LastFrameId}, {nameof(RttInMs)}: {RttInMs}, {nameof(ReceivePacketCount)}: {ReceivePacketCount}, {nameof(SendPacketCount)}: {SendPacketCount}, {nameof(ReceiveSize)}: {ReceiveSize}, {nameof(SendSize)}: {SendSize}";
        }
    }
}
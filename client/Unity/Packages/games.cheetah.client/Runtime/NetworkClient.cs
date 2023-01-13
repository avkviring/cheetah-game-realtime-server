using System;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Internal;
using Games.Cheetah.Client.Internal.FFI;
using Games.Cheetah.Client.Logger;
using Games.Cheetah.Client.Types.Command;
using Games.Cheetah.Client.Types.Field;
using Games.Cheetah.Client.Types.Network;
using Games.Cheetah.Client.Types.Object;
using UnityEngine;

namespace Games.Cheetah.Client
{
    /// <summary>
    /// Клиент Relay сервера
    /// </summary>
    public class NetworkClient
    {
        private static NetworkClient current;

        public readonly CodecRegistry CodecRegistry;
        internal readonly ushort Id;
        private bool enableClientLog = true;
        private ChannelType currentChannelType;
        private byte currentChannelGroup;
        private NetworkBuffer buffer;
        internal static S2CCommand[] s2cCommands = new S2CCommand[1024];
        internal ushort s2cCommandsCount;
        public Writer Writer { get; }
        public Reader Reader { get; }

        static NetworkClient()
        {
            NetworkClientLogs.Init();
        }

        internal readonly IFFI ffi;

        public NetworkClient(
            string serverUdpHost,
            ushort serverUdpPort,
            uint memberId,
            ulong roomId,
            byte[] privateUserKey,
            CodecRegistry codecRegistry) : this(new FFIImpl(), serverUdpHost, serverUdpPort, memberId, roomId, privateUserKey, codecRegistry)
        {
        }

        internal NetworkClient(
            IFFI ffi,
            string serverUdpHost,
            ushort serverUdpPort,
            uint memberId,
            ulong roomId,
            byte[] privateUserKey,
            CodecRegistry codecRegistry)
        {
            this.ffi = ffi;
            NetworkClientLogs.CollectLogs(false); // очищаем логи с предыдущего клиента
            CodecRegistry = codecRegistry;

            var userPrivateKey = new NetworkBuffer(privateUserKey);
            ResultChecker.Check(ffi.CreateClient(
                serverUdpHost + ":" + serverUdpPort,
                (ushort)memberId,
                roomId,
                ref userPrivateKey,
                0,
                out Id));

            Writer = new Writer(ffi, CodecRegistry, Id);
            Reader = new Reader(this, CodecRegistry);
        }

        /// <summary>
        /// Отключить клиентские логи
        /// </summary>
        public void DisableClientLog()
        {
            enableClientLog = false;
        }


        /// <summary>
        /// Обновление состояние. Получение сетевых команд.
        /// </summary>
        public void Update()
        {
            unsafe
            {
                current = this;
                fixed (S2CCommand* commands = s2cCommands)
                {
                    ResultChecker.Check(ffi.Receive(Id, commands, ref s2cCommandsCount));
                }
                Reader.Update();
                NetworkClientLogs.CollectLogs(enableClientLog);
            }
        }


        public ConnectionStatus GetConnectionStatus()
        {
            ResultChecker.Check(ffi.GetConnectionStatus(Id, out var connectionStatus));
            return connectionStatus;
        }

        public Statistics GetStatistics()
        {
            ResultChecker.Check(ffi.GetStatistics(Id, out var statistics));
            return statistics;
        }


        /// <summary>
        /// Создать объект, принадлежащий пользователю
        /// </summary>
        public NetworkObjectBuilder NewObjectBuilder(ushort template, ulong accessGroup)
        {
            return new NetworkObjectBuilder(template, accessGroup, this);
        }

        public void OnException(Exception e)
        {
            Debug.LogException(e);
        }

        /// <summary>
        /// Зайти в комнату, после этой команды сервер начнет загрузку игровых объектов
        /// </summary>
        public void AttachToRoom()
        {
            ResultChecker.Check(ffi.AttachToRoom(Id));
        }

        /// <summary>
        /// Выйти из комнаты, после этого сервер не будет посылать команды на текущий клиент
        /// </summary>
        public void DetachFromRoom()
        {
            ResultChecker.Check(ffi.DetachFromRoom(Id));
        }

        /// <summary>
        /// Отсоединиться от сервера и удалить информацию о текущем клиенте, после этого методами RelayClient пользоваться нельзя
        /// </summary>
        public void Dispose()
        {
            ResultChecker.Check(ffi.DestroyClient(Id));
        }

        /// <summary>
        /// Получить серверное время (монотонно возрастающее, отсчет от времени запуска сервера)
        /// </summary>
        /// <returns></returns>
        /// <exception cref="ServerTimeNotDefinedException"></exception>
        public ulong GetServerTimeInMs()
        {
            ResultChecker.Check(ffi.GetServerTime(Id, out var time));
            if (time == 0)
            {
                throw new ServerTimeNotDefinedException();
            }

            return time;
        }


        /// <summary>
        /// Установить канал отправки все последующих команд
        /// </summary>
        /// <param name="channelType">тип канала</param>
        /// <param name="group">группа, для групповых каналов, для остальных игнорируется</param>
        public void SetChannelType(ChannelType channelType, byte group)
        {
            if (currentChannelType == channelType && currentChannelGroup == group)
            {
                return;
            }

            currentChannelType = channelType;
            currentChannelGroup = group;
            ResultChecker.Check(ffi.SetChannelType(Id, channelType, group));
        }


        /// <summary>
        /// Сброс эмуляции параметров сети
        /// </summary>
        public void ResetEmulation()
        {
            ResultChecker.Check(ffi.ResetEmulation(Id));
        }

        /// <summary>
        /// Задать параметры эмуляции RTT
        /// Подробнее смотрите в документации проекта
        /// </summary>
        public void SetRttEmulation(ulong rttInMs, double rttDispersion)
        {
            ResultChecker.Check(ffi.SetRttEmulation(Id, rttInMs, rttDispersion));
        }

        /// <summary>
        /// Задать параметры эмуляции потери пакетов
        /// Подробнее смотрите в документации проекта
        /// </summary>
        public void SetDropEmulation(double dropProbability, ulong dropTimeInMs)
        {
            ResultChecker.Check(ffi.SetDropEmulation(Id, dropProbability, dropTimeInMs));
        }
    }

    public class ServerTimeNotDefinedException : Exception
    {
    }
}
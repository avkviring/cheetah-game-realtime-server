using System;
using System.Collections.Generic;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Internal;
using Games.Cheetah.Client.Internal.Plugin;
using Games.Cheetah.Client.Internal.Plugin.Routers.FFI;
using Games.Cheetah.Client.Logger;
using Games.Cheetah.Client.ServerAPI;
using Games.Cheetah.Client.ServerAPI.FFI;
using Games.Cheetah.Client.Types;
using UnityEngine;

namespace Games.Cheetah.Client
{
    /// <summary>
    /// Клиент Relay сервера
    /// </summary>
    public class CheetahClient
    {
        public readonly CodecRegistry CodecRegistry;
        internal readonly ushort Id;
        public event Action BeforeUpdateHook;
        private readonly Dictionary<Type, object> plugins = new();
        private readonly CheetahObjectsCreateInfo objectsCreateInfo;
        private bool enableClientLog = true;
        private ChannelType currentChannelType;
        private byte currentChannelGroup;
        private CheetahBuffer buffer;
        public Writer Writer { get; }
        public Reader Reader { get; }

        static CheetahClient()
        {
            LoggerGateway.Init();
        }

        internal IServerAPI serverAPI;

        public CheetahClient(
            string serverUdpHost,
            ushort serverUdpPort,
            uint memberId,
            ulong roomId,
            byte[] privateUserKey,
            CodecRegistry codecRegistry) : this(new FFIServerAPI(), serverUdpHost, serverUdpPort, memberId, roomId, privateUserKey, codecRegistry)
        {
        }

        internal CheetahClient(
            IServerAPI serverAPI,
            string serverUdpHost,
            ushort serverUdpPort,
            uint memberId,
            ulong roomId,
            byte[] privateUserKey,
            CodecRegistry codecRegistry)
        {
            this.serverAPI = serverAPI;
            LoggerGateway.CollectLogs(false); // очищаем логи с предыдущего клиента
            CodecRegistry = codecRegistry;

            var userPrivateKey = new CheetahBuffer(privateUserKey);
            ResultChecker.Check(serverAPI.Client.CreateClient(
                serverUdpHost + ":" + serverUdpPort,
                (ushort)memberId,
                roomId,
                ref userPrivateKey,
                0,
                out Id));
            objectsCreateInfo = GetPlugin<CheetahObjectsCreateInfo>();
            GetPlugin<LongCommandRouter>();
            Writer = new Writer(serverAPI, CodecRegistry, Id);
            Reader = new Reader(this);
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
            BeforeUpdateHook?.Invoke();
            ResultChecker.Check(serverAPI.Client.Receive(Id));
            LoggerGateway.CollectLogs(enableClientLog);
        }

        public CheetahClientConnectionStatus GetConnectionStatus()
        {
            ResultChecker.Check(serverAPI.Client.GetConnectionStatus(Id, out var connectionStatus));
            return connectionStatus;
        }

        public CheetahClientStatistics GetStatistics()
        {
            ResultChecker.Check(serverAPI.Client.GetStatistics(Id, out var statistics));
            return statistics;
        }

        public T GetPlugin<T>() where T : Plugin, new()
        {
            if (plugins.TryGetValue(typeof(T), out var plugin))
            {
                return (T)plugin;
            }

            var newPlugin = new T();
            newPlugin.Init(this);
            plugins.Add(typeof(T), newPlugin);
            return newPlugin;
        }


        /// <summary>
        /// Создать объект, принадлежащий пользователю
        /// </summary>
        public CheetahObjectBuilder NewObjectBuilder(ushort template, ulong accessGroup)
        {
         
            return new CheetahObjectBuilder(template, accessGroup, objectsCreateInfo, this);
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
            ResultChecker.Check(serverAPI.Client.AttachToRoom(Id));
        }

        /// <summary>
        /// Выйти из комнаты, после этого сервер не будет посылать команды на текущий клиент
        /// </summary>
        public void DetachFromRoom()
        {
            ResultChecker.Check(serverAPI.Client.DetachFromRoom(Id));
        }

        /// <summary>
        /// Отсоединиться от сервера и удалить информацию о текущем клиенте, после этого методами RelayClient пользоваться нельзя
        /// </summary>
        public void Dispose()
        {
            ResultChecker.Check(serverAPI.Client.DestroyClient(Id));
        }

        /// <summary>
        /// Получить серверное время (монотонно возрастающее, отсчет от времени запуска сервера)
        /// </summary>
        /// <returns></returns>
        /// <exception cref="ServerTimeNotDefinedException"></exception>
        public ulong GetServerTimeInMs()
        {
            ResultChecker.Check(serverAPI.Client.GetServerTime(Id, out var time));
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
            ResultChecker.Check(serverAPI.Client.SetChannelType(Id, channelType, group));
        }


        /// <summary>
        /// Сброс эмуляции параметров сети
        /// </summary>
        public void ResetEmulation()
        {
            ResultChecker.Check(serverAPI.Client.ResetEmulation(Id));
        }

        /// <summary>
        /// Задать параметры эмуляции RTT
        /// Подробнее смотрите в документации проекта
        /// </summary>
        public void SetRttEmulation(ulong rttInMs, double rttDispersion)
        {
            ResultChecker.Check(serverAPI.Client.SetRttEmulation(Id, rttInMs, rttDispersion));
        }

        /// <summary>
        /// Задать параметры эмуляции потери пакетов
        /// Подробнее смотрите в документации проекта
        /// </summary>
        public void SetDropEmulation(double dropProbability, ulong dropTimeInMs)
        {
            ResultChecker.Check(serverAPI.Client.SetDropEmulation(Id, dropProbability, dropTimeInMs));
        }
    }

    public class ServerTimeNotDefinedException : Exception
    {
    }
}
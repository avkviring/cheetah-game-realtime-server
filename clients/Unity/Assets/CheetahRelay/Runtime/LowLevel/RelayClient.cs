using System;
using CheetahRelay.Runtime.LowLevel.External;
using CheetahRelay.Runtime.LowLevel.Listener;
using CheetahRelay.Runtime.LowLevel.Object;

namespace CheetahRelay.Runtime.LowLevel
{
    /**
     * Клиент для взаимодействия с Relay сервером
     *
     * - методы для отправки данных можно вызывать до события [IRelayClientListener:onConnect]
     * - методы получения данных можно вызывать в любое время
     * - после события [IRelayClientListener:onDisconnect] вызовы методов отправки данных будут проигнорированы
     */
    public class RelayClient
    {
        private readonly ushort _clientId;
        private readonly IServerCommandListener _serverCommandListener;
        private readonly IRelayClientListener _relayClientListener;

        private Command _command;
        
        /**
         * true - если сетевое соединение установлено
         */
        private bool _connected;
        /**
         * true - если был факт разрыва соединения (после коннекта или попытки коннекта)
         */
        private bool _disconnected;
        
        private readonly RelayGameObjectBuilder _gameObjectBuilder = new RelayGameObjectBuilder();
        

        public RelayClient(
            string serverAddress, 
            string roomHash, 
            string clientHash, 
            IRelayClientListener relayClientListener,
            IServerCommandListener serverCommandListener)
        {
            _serverCommandListener = serverCommandListener;
            _relayClientListener = relayClientListener;

            _clientId = Externals.CreateClient(serverAddress, roomHash, clientHash);
            _gameObjectBuilder.SetClientId(_clientId);
        }

        private void OnChangeNetworkStatus(NetworkStatus networkStatus)
        {
            switch (networkStatus)
            {
                case NetworkStatus.None:
                    break;
                case NetworkStatus.Connecting:
                    break;
                case NetworkStatus.OnLine:
                    OnConnected();
                    break;
                case NetworkStatus.Disconnected:
                    OnDisconnected();
                    break;
                default:
                    throw new ArgumentOutOfRangeException(nameof(networkStatus), networkStatus, null);
            }
        }

        private void OnConnected()
        {
            if (_connected) return;
            _connected = true;
            _relayClientListener.OnConnected();
        }

        private void OnDisconnected()
        {
            if (_disconnected) return;
            _disconnected = true;
            _connected = false;
            _relayClientListener.OnDisconnect();
        }


        private void ServerCommand(in Command command)
        {
            switch (command.commandTypeS2C)
            {
                case S2CCommandType.Upload:
                    _serverCommandListener.OnObjectUploaded(
                        in command.objectId,
                        in command.structures,
                        in command.longCounters,
                        in command.doubleCounters);
                    break;
                case S2CCommandType.SetLongCounter:
                    _serverCommandListener.OnLongCounterUpdated(in command.objectId, command.fieldId, command.longValue);
                    break;
                case S2CCommandType.SetFloatCounter:
                    _serverCommandListener.OnFloatCounterUpdated(in command.objectId, command.fieldId, command.floatValue);
                    break;
                case S2CCommandType.Structure:
                    _serverCommandListener.OnStructureUpdated(in command.objectId, command.fieldId, in command.structureData);
                    break;
                case S2CCommandType.Event:
                    _serverCommandListener.OnEvent(in command.objectId, command.fieldId, in command.eventData);
                    break;
                case S2CCommandType.Unload:
                    _serverCommandListener.OnObjectUnloaded(in command.objectId);
                    break;
                default:
                    throw new ArgumentOutOfRangeException();
            }
        }

        /**
         * Цикл обновления клиента
         */
        public void Update()
        {
            if (_disconnected)
            {
                return;
            }

            Externals.GetConnectionStatus(_clientId, this.OnChangeNetworkStatus, this.OnDisconnected);
            Externals.ReceiveCommandsFromServer(_clientId, this.ServerCommand, this.OnDisconnected);
        }
        


        /**
         * Получить builder для создания объекта
         * 
         * - не поддерживается одновременное создание нескольких объектов
         * - не поддерживается многопоточная работа
         * 
         */
        public IRelayGameObjectBuilder GetGameObjectBuilder()
        {
            _gameObjectBuilder.PrepareForNewObject(!_disconnected);
            return _gameObjectBuilder;
        }


        public void SendEventToServer(in RelayGameObjectId objectId, ushort eventId, in Bytes data)
        {
            if (_disconnected)
            {
                return;
            }

            _command.commandTypeC2S = C2SCommandType.Event;
            _command.objectId = objectId;
            _command.fieldId = eventId;
            _command.eventData = data;
            Externals.SendCommandToServer(_clientId, in _command);
        }

        public void UpdateStructureOnServer(in RelayGameObjectId objectId, ushort structureId, in Bytes data)
        {
            if (_disconnected)
            {
                return;
            }

            _command.commandTypeC2S = C2SCommandType.Structure;
            _command.objectId = objectId;
            _command.fieldId = structureId;
            _command.structureData = data;
            Externals.SendCommandToServer(_clientId, in _command);
        }

        public void IncrementLongCounterOnServer(in RelayGameObjectId objectId, ushort counterId, long increment)
        {
            if (_disconnected)
            {
                return;
            }

            _command.commandTypeC2S = C2SCommandType.IncrementLongCounter;
            _command.objectId = objectId;
            _command.fieldId = counterId;
            _command.longValue = increment;
            Externals.SendCommandToServer(_clientId, in _command);
        }

        public void IncrementFloatCounterOnServer(in RelayGameObjectId objectId, ushort counterId, double increment)
        {
            if (_disconnected)
            {
                return;
            }

            _command.commandTypeC2S = C2SCommandType.IncrementFloatCounter;
            _command.objectId = objectId;
            _command.fieldId = counterId;
            _command.floatValue = increment;
            Externals.SendCommandToServer(_clientId, in _command);
        }

        public void SetLongCounterOnServer(in RelayGameObjectId objectId, ushort counterId, long value)
        {
            if (_disconnected)
            {
                return;
            }

            _command.commandTypeC2S = C2SCommandType.SetLongCounter;
            _command.objectId = objectId;
            _command.fieldId = counterId;
            _command.longValue = value;
            Externals.SendCommandToServer(_clientId, in _command);
        }

        public void SetFloatCounterOnServer(in RelayGameObjectId objectId, ushort counterId, double value)
        {
            if (_disconnected)
            {
                return;
            }

            _command.commandTypeC2S = C2SCommandType.SetFloatCounter;
            _command.objectId = objectId;
            _command.fieldId = counterId;
            _command.floatValue = value;
            Externals.SendCommandToServer(_clientId, in _command);
        }

        public void Close()
        {
            Externals.DestroyClient(_clientId);
            OnDisconnected();
        }
    }
}
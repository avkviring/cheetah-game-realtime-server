using System;
using CheetahRelay.Runtime.LowLevel.External;

namespace CheetahRelay.Runtime.LowLevel.Object
{
    public interface IRelayGameObjectBuilder
    {
        IRelayGameObjectBuilder SetAccessGroup(ulong accessGroup);
        IRelayGameObjectBuilder AddStruct(ushort fieldId, in Bytes data);
        IRelayGameObjectBuilder AddLongCounter(ushort fieldId, long value);
        IRelayGameObjectBuilder AddFloatCounter(ushort fieldId, double value);

        /**
         * Создать объект и отправить его на сервер        
         *
         * - повторный вызов приведет к ошибке
         */
        RelayGameObjectId BuildAndSendToServer();
    }

    public class RelayGameObjectBuilder : IRelayGameObjectBuilder
    {
        private ushort _clientId;
        private Command _command;
        private bool _built;
        private uint _nextGameObjectId;
        private bool _allowSend;

        public void SetClientId(ushort clientId)
        {
            this._clientId = clientId;
        }

        public void PrepareForNewObject(bool allowSend)
        {
            _allowSend = allowSend;
            _built = false;
            _command.objectId.id = _nextGameObjectId;
            _command.objectId.client = 0;
            _command.objectId.type = RelayGameObjectIdType.Current;
            _command.structures.count = 0;
            _command.longCounters.count = 0;
            _command.doubleCounters.count = 0;

            _nextGameObjectId++;
        }

        public IRelayGameObjectBuilder SetAccessGroup(ulong accessGroup)
        {
            _command.accessGroup = accessGroup;
            return this;
        }

        public IRelayGameObjectBuilder AddStruct(ushort fieldId, in Bytes data)
        {
            var index = _command.structures.count;
            if (index == Command.BufferSize)
            {
                throw new OverflowException();
            }

            unsafe
            {
                _command.structures.fields[index] = fieldId;
                _command.structures.sizes[index] = data.size;
                var offset = index * Command.BufferSize;
                for (var i = 0; i < data.size; i++)
                {
                    _command.structures.values[offset + i] = data.values[i];
                }
            }

            _command.structures.count++;
            return this;
        }

        public IRelayGameObjectBuilder AddLongCounter(ushort fieldId, long value)
        {
            var index = _command.longCounters.count;
            if (index == Command.BufferSize)
            {
                throw new OverflowException();
            }

            unsafe
            {
                _command.longCounters.fields[index] = fieldId;
                _command.longCounters.values[index] = value;
            }

            _command.longCounters.count++;
            return this;
        }

        public IRelayGameObjectBuilder AddFloatCounter(ushort fieldId, double value)
        {
            var index = _command.doubleCounters.count;
            if (index == Command.BufferSize)
            {
                throw new OverflowException();
            }

            unsafe
            {
                _command.doubleCounters.fields[index] = fieldId;
                _command.doubleCounters.values[index] = value;
            }

            _command.doubleCounters.count++;
            return this;
        }


        public RelayGameObjectId BuildAndSendToServer()
        {
            if (_built)
            {
                throw new Exception("BuildAndSendToServer already invoked");
            }

            _built = true;
            if (_allowSend)
            {
                Externals.SendCommandToServer(_clientId, in _command);
            }
            return _command.objectId;
        }
    }
}
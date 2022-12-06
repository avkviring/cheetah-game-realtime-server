using System;
using System.Text;
using Games.Cheetah.Client.Internal.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal
{
    public static class ResultChecker
    {
        public static void Check(byte code)
        {
            switch (code)
            {
                case 1: throw new CheetahClientRegistryMutexError(GetLastError());
                case 2: throw new ClientNotFoundError(GetLastError());
                case 3: throw new ConnectionStatusMutexError(GetLastError());
                case 4: throw new SendTaskError(GetLastError());
                case 5: throw new CreateClientError(GetLastError());
            }
        }

        private static string GetLastError()
        {
            var buffer = new CheetahBuffer();
            ClientFFI.GetLastErrorMsg(ref buffer);
            var bytes = new byte[buffer.size];
            for (var i = 0; i < buffer.size; i++)
            {
                unsafe
                {
                    bytes[i] = buffer.values[i];
                }
            }

            return Encoding.UTF8.GetString(bytes);
        }
    }

    public class CheetahClientRegistryMutexError : Exception
    {
        public CheetahClientRegistryMutexError(string msg) : base(msg)
        {
        }
    }

    public class ClientNotFoundError : Exception
    {
        public ClientNotFoundError(string msg) : base(msg)
        {
        }
    }

    public class ConnectionStatusMutexError : Exception
    {
        public ConnectionStatusMutexError(string msg) : base(msg)
        {
        }
    }

    public class SendTaskError : Exception
    {
        public SendTaskError(string msg) : base(msg)
        {
        }
    }

    public class CreateClientError : Exception
    {
        public CreateClientError(string msg) : base(msg)
        {
        }
    }
}
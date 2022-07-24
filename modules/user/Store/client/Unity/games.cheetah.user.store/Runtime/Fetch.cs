using System;
using System.Threading.Tasks;
using Grpc.Core;
using Cheetah.Platform;
using Cheetah.UserStore.GRPC;

namespace Cheetah.UserStore
{
    /// <summary>Объект, предоставляющий доступ к функциям
    /// доступа к данным в пользовательском хранилище.</summary>
    public class Fetch
    {
        private readonly ClusterConnector _connector;

        public Fetch(ClusterConnector connector)
        {
            _connector = connector;
        }


        /// <summary>
        /// <para>Извлекает значение поля <paramref name="fieldName"/> из хранилища.</para>
        /// </summary>
        /// <returns>Возвращает значение поля или null, если оно не найдено</returns>
        /// <exception cref="UserStoreException"/>
        public async Task<double?> TryGetDouble(string fieldName)
        {
            var request = new FetchDoubleRequest { FieldName = fieldName };
            var result = await ExecuteFetch(async client =>
            {
                return await client.DoubleAsync(request);
            });

            switch (result.ResultCase)
            {
                case FetchDoubleReply.ResultOneofCase.Value:
                    return result.Value;
                default:
                    throw UserStoreException.FromGrpcStatus(result.Status);
            }
        }


        /// <summary>
        /// <para>Извлекает значение поля <paramref name="fieldName"/> из хранилища.</para>
        /// </summary>
        /// <returns>Возвращает значение поля или null, если оно не найдено</returns>
        /// <exception cref="UserStoreException"/>
        public async Task<long?> TryGetLong(string fieldName)
        {
            var request = new FetchLongRequest { FieldName = fieldName };
            var result = await ExecuteFetch(async client =>
            {
                return await client.LongAsync(request);
            });

            switch (result.ResultCase)
            {
                case FetchLongReply.ResultOneofCase.Value:
                    return result.Value;
                default:
                    throw UserStoreException.FromGrpcStatus(result.Status);
            }
        }

        /// <summary>
        /// <para>Извлекает значение поля <paramref name="fieldName"/> из хранилища.</para>
        /// </summary>
        /// <returns>Возвращает значение поля или null, если оно не найдено</returns>
        /// <exception cref="UserStoreException"/>
        public async Task<string> TryGetString(string fieldName)
        {
            var request = new FetchStringRequest { FieldName = fieldName };
            var result = await ExecuteFetch(async client =>
            {
                return await client.StringAsync(request);
            });

            switch (result.ResultCase)
            {
                case FetchStringReply.ResultOneofCase.Value:
                    return result.Value;
                default:
                    throw UserStoreException.FromGrpcStatus(result.Status);
            }
        }

        private async Task<T> ExecuteFetch<T>(Func<GRPC.Fetch.FetchClient, Task<T>> func)
        {
            return await _connector.DoRequest(async channel =>
            {
                var client = new GRPC.Fetch.FetchClient(channel);
                try
                {
                    return await func(client);
                }
                catch (RpcException e)
                {
                    throw UserStoreException.FromRpcException(e);
                }
            });
        }
    }
}

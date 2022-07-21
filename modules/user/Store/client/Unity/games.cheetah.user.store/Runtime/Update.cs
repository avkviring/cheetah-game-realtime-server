using System;
using System.Threading.Tasks;
using Cheetah.Platform;
using Cheetah.User.Store.GRPC;
using Grpc.Core;

namespace Cheetah.User.Store
{
    /// <summary>Объект, предоставляющий доступ к функциям обновления
    /// пользовательского хранилища.</summary>
    public class Update
    {
        private readonly ClusterConnector _connector;

        public Update(ClusterConnector connector)
        {
            _connector = connector;
        }

        /// <summary>
        /// <para>Устанавливает значение поля <paramref name="fieldName"/>.</para>
        /// <para>Если поле не существует, то оно будет создано.</para>
        /// </summary>
        /// <exception cref="UserStoreException"/>
        public async void SetLong(string fieldName, long value)
        {
            var request = new SetLongRequest { FieldName = fieldName, Value = value };
            await ExecuteUpdate(async client => { await client.SetLongAsync(request); });
        }

        /// <summary>
        /// <para>Устанавливает значение поля <paramref name="fieldName"/>.</para>
        /// <para>Если поле не существует, то оно будет создано.</para>
        /// </summary>
        /// <exception cref="UserStoreException"/>
        public async void SetDouble(string fieldName, double value)
        {
            var request = new SetDoubleRequest { FieldName = fieldName, Value = value };
            await ExecuteUpdate(async client => { await client.SetDoubleAsync(request); });
        }

        /// <summary>
        /// <para>Устанавливает значение поля <paramref name="fieldName"/>.</para>
        /// <para>Если поле не существует, то оно будет создано.</para>
        /// </summary>
        /// <exception cref="UserStoreException"/>
        public async void SetString(string fieldName, string value)
        {
            var request = new SetStringRequest { FieldName = fieldName, Value = value };
            await ExecuteUpdate(async client => { await client.SetStringAsync(request); });
        }

        /// <summary>
        /// <para>Добавляет к значению поля <paramref name="fieldName"/> значение <paramref name="value"/>.</para>
        /// </summary>
        /// <remarks>В случае если поле не найдено метод не возвращает ошибку.</remarks>
        /// <exception cref="UserStoreException"/>
        public async void IncrementLong(string fieldName, long value)
        {
            var request = new SetLongRequest { FieldName = fieldName, Value = value };
            await ExecuteUpdate(async client => { await client.IncrementLongAsync(request); });
        }

        /// <summary>
        /// <para>Добавляет к значению поля <paramref name="fieldName"/> значение <paramref name="value"/>.</para>
        /// </summary>
        /// <remarks>В случае если поле не найдено метод не возвращает ошибку.</remarks>
        /// <exception cref="UserStoreException"/>
        public async void IncrementDouble(string fieldName, double value)
        {
            var request = new SetDoubleRequest { FieldName = fieldName, Value = value };
            await ExecuteUpdate(async client => { await client.IncrementDoubleAsync(request); });
        }

        private async Task ExecuteUpdate(Func<User.Store.GRPC.Update.UpdateClient, Task> func)
        {
            await _connector.DoRequest(async channel =>
            {
                var client = new User.Store.GRPC.Update.UpdateClient(channel);
                try
                {
                    await func(client);
                }
                catch (RpcException e)
                {
                    throw UserStoreException.FromRpcException(e);
                }
            });
        }
    }
}
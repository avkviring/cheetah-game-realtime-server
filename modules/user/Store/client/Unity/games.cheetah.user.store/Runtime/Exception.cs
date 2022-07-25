using System;
using Grpc.Core;

namespace Cheetah.User.Store
{
    /// <summary>Перечисление возможных ошибочных ситуаций.</summary>
    public enum Status
    {
        FieldNotFound,
        PermissionDenied,
        TypeError,
        InternalServerError
    }

    static class Messages
    {
        public const string TYPE_ERROR = "The received value was of incorrect type";
    }

    /// <summary>Исключение, бросаемое в случае ошибки сервиса хранения
    /// пользовательских данных.</summary>
    public class UserStoreException : Exception
    {
        /// <summary>Статус, уточняющий природу ошибки.</summary>
        public Status Status { get; }

        internal UserStoreException(string message, Exception source, Status status) : base(message, source)
        {
            Status = status;
        }

        internal static UserStoreException FromRpcException(RpcException e)
        {
            switch (e.StatusCode)
            {
                case StatusCode.PermissionDenied:
                    return new UserStoreException("Permission denied", e, Status.PermissionDenied);
                default:
                    return new UserStoreException("Internal server error", e, Status.InternalServerError);
            }
        }

        internal static UserStoreException FromGrpcStatus(User.Store.GRPC.Status status)
        {
            switch (status)
            {
                case User.Store.GRPC.Status.FieldNotFound:
                    return new UserStoreException("Field not found", null, Status.FieldNotFound);
                default:
                    return new UserStoreException("Internal server error", null, Status.InternalServerError);
            }
        }
    }
}
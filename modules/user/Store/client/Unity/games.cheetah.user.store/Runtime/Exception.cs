using System;
using Grpc.Core;
using Cheetah.UserStore.GRPC;

namespace Cheetah.UserStore
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

        static internal UserStoreException FromRpcException(RpcException e)
        {
            switch (e.StatusCode)
            {
                case StatusCode.PermissionDenied:
                    return new UserStoreException("Permission denied", e, Status.PermissionDenied);
                default:
                    return new UserStoreException("Internal server error", e, Status.InternalServerError);
            }
        }

        static internal UserStoreException FromGrpcStatus(GRPC.Status status)
        {
            switch (status)
            {
                case GRPC.Status.FieldNotFound:
                    return new UserStoreException("Field not found", null, Status.FieldNotFound);
                default:
                    return new UserStoreException("Internal server error", null, Status.InternalServerError);
            }
        }
    }
}
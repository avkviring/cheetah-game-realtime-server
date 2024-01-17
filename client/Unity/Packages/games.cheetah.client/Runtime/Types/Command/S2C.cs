using System.Runtime.InteropServices;
using Games.Cheetah.Client.Types.Field;
using Games.Cheetah.Client.Types.Object;
using UnityEngine.Serialization;

namespace Games.Cheetah.Client.Types.Command
{
    [StructLayout(LayoutKind.Sequential)]
    public struct S2CCommand
    {
        public CommandType commandType;
        public S2CCommandUnion commandUnion;

        public override string ToString()
        {
            return $"{nameof(commandType)}: {commandType} " + (commandType switch
            {
                CommandType.CreateGameObject => commandUnion.createObject.ToString(),
                CommandType.CreatedGameObject => commandUnion.createdObject.ToString(),
                CommandType.SetLong => commandUnion.setLong.ToString(),
                CommandType.SetDouble => commandUnion.setDouble.ToString(),
                CommandType.SetStructure => commandUnion.binaryField.ToString(),
                CommandType.SendEvent => commandUnion.setEvent.ToString(),
                CommandType.DeleteObject => commandUnion.deleteObject.ToString(),
                CommandType.DeleteField => commandUnion.deleteField.ToString(),
                _ => ""
            });
        }
    }


    [StructLayout(LayoutKind.Explicit)]
    public struct S2CCommandUnion
    {
        [FieldOffset(0)] public S2CCommands.CreateObject createObject;
        [FieldOffset(0)] public S2CCommands.CreatedObject createdObject;
        [FieldOffset(0)] public S2CCommands.SetLong setLong;
        [FieldOffset(0)] public S2CCommands.SetDouble setDouble;
        [FieldOffset(0)] public S2CCommands.BinaryField binaryField;
        [FieldOffset(0)] public S2CCommands.Event setEvent;
        [FieldOffset(0)] public S2CCommands.DeleteObject deleteObject;
        [FieldOffset(0)] public S2CCommands.DeleteField deleteField;
        [FieldOffset(0)] public S2CCommands.MemberConnected memberConnected;
        [FieldOffset(0)] public S2CCommands.MemberDisconnected memberDisconnected;
    }

    public interface S2CCommands
    {
        [StructLayout(LayoutKind.Sequential)]
        public struct CreateObject
        {
            public NetworkObjectId objectId;
            public ushort template;
            public ulong accessGroup;

            public CreateObject(NetworkObjectId id, ushort template, ulong accessGroup)
            {
                objectId = id;
                this.template = template;
                this.accessGroup = accessGroup;
            }

            public override string ToString()
            {
                return
                    $"{nameof(objectId)}: {objectId}, {nameof(template)}: {template}, {nameof(accessGroup)}: {accessGroup}";
            }
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct CreatedObject
        {
            public NetworkObjectId objectId;

            public CreatedObject(NetworkObjectId id)
            {
                objectId = id;
            }

            public override string ToString()
            {
                return $"{nameof(objectId)}: {objectId}";
            }
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct DeleteObject
        {
            public NetworkObjectId objectId;

            public DeleteObject(NetworkObjectId id)
            {
                objectId = id;
            }
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct Event
        {
            public NetworkObjectId objectId;
            public ushort fieldId;
            public NetworkBuffer eventData;

            public Event(NetworkObjectId objectId, FieldId.Event fieldId, NetworkBuffer eventData)
            {
                this.objectId = objectId;
                this.fieldId = fieldId.Id;
                this.eventData = eventData;
            }


            public override string ToString()
            {
                return
                    $"{nameof(objectId)}: {objectId}, {nameof(fieldId)}: {fieldId}, {nameof(eventData)}: {eventData}";
            }
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct DeleteField
        {
            public NetworkObjectId objectId;
            public ushort fieldId;
            public FieldType fieldType;
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct SetLong
        {
            public NetworkObjectId objectId;
            public ushort fieldId;
            public long value;

            public SetLong(NetworkObjectId id, FieldId.Long field, long value)
            {
                objectId = id;
                fieldId = field.Id;
                this.value = value;
            }

            public override string ToString()
            {
                return $"{nameof(objectId)}: {objectId}, {nameof(fieldId)}: {fieldId}, {nameof(value)}: {value}";
            }
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct SetDouble
        {
            public NetworkObjectId objectId;
            public ushort fieldId;
            public double value;

            public SetDouble(NetworkObjectId id, FieldId.Double field, float value)
            {
                objectId = id;
                fieldId = field.Id;
                this.value = value;
            }


            public override string ToString()
            {
                return $"{nameof(objectId)}: {objectId}, {nameof(fieldId)}: {fieldId}, {nameof(value)}: {value}";
            }
        }


        [StructLayout(LayoutKind.Sequential)]
        public struct BinaryField
        {
            public NetworkObjectId objectId;
            public ushort fieldId;
            public NetworkBuffer value;

            public BinaryField(NetworkObjectId id, FieldId.Structure field, NetworkBuffer value)
            {
                objectId = id;
                fieldId = field.Id;
                this.value = value;
            }
            
            public BinaryField(NetworkObjectId id, FieldId.Items field, NetworkBuffer value)
            {
                objectId = id;
                fieldId = field.Id;
                this.value = value;
            }


            public override string ToString()
            {
                return $"{nameof(objectId)}: {objectId}, {nameof(fieldId)}: {fieldId}, {nameof(value)}: {value}";
            }
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct MemberConnected
        {
            public ushort MemberId;

            public MemberConnected(ushort memberId)
            {
                MemberId = memberId;
            }

            public override string ToString()
            {
                return $"{nameof(MemberId)}: {MemberId}";
            }
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct MemberDisconnected
        {
            public ushort MemberId;

            public MemberDisconnected(ushort memberId)
            {
                MemberId = memberId;
            }

            public override string ToString()
            {
                return $"{nameof(MemberId)}: {MemberId}";
            }
        }
    }

    public enum CommandType
    {
        CreateGameObject = 0,
        CreatedGameObject,
        SetLong,
        IncrementLong,
        SetDouble,
        IncrementDouble,
        SetStructure,
        SendEvent,
        TargetEvent,
        DeleteObject,
        AttachToRoom,
        DetachFromRoom,
        DeleteField,
        MemberConnected,
        MemberDisconnected,
        AddItem
    }
}
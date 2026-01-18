impl serde::Serialize for AoiSyncDelta {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.uuid != 0 {
            len += 1;
        }
        if self.attrs.is_some() {
            len += 1;
        }
        if self.skill_effects.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.AoiSyncDelta", len)?;
        if self.uuid != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("uuid", ToString::to_string(&self.uuid).as_str())?;
        }
        if let Some(v) = self.attrs.as_ref() {
            struct_ser.serialize_field("attrs", v)?;
        }
        if let Some(v) = self.skill_effects.as_ref() {
            struct_ser.serialize_field("skillEffects", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AoiSyncDelta {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "uuid",
            "attrs",
            "skill_effects",
            "skillEffects",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Uuid,
            Attrs,
            SkillEffects,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "uuid" => Ok(GeneratedField::Uuid),
                            "attrs" => Ok(GeneratedField::Attrs),
                            "skillEffects" | "skill_effects" => Ok(GeneratedField::SkillEffects),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AoiSyncDelta;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.AoiSyncDelta")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AoiSyncDelta, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut uuid__ = None;
                let mut attrs__ = None;
                let mut skill_effects__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Uuid => {
                            if uuid__.is_some() {
                                return Err(serde::de::Error::duplicate_field("uuid"));
                            }
                            uuid__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Attrs => {
                            if attrs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("attrs"));
                            }
                            attrs__ = map_.next_value()?;
                        }
                        GeneratedField::SkillEffects => {
                            if skill_effects__.is_some() {
                                return Err(serde::de::Error::duplicate_field("skillEffects"));
                            }
                            skill_effects__ = map_.next_value()?;
                        }
                    }
                }
                Ok(AoiSyncDelta {
                    uuid: uuid__.unwrap_or_default(),
                    attrs: attrs__,
                    skill_effects: skill_effects__,
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.AoiSyncDelta", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AoiSyncToMeDelta {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.base_delta.is_some() {
            len += 1;
        }
        if self.uuid != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.AoiSyncToMeDelta", len)?;
        if let Some(v) = self.base_delta.as_ref() {
            struct_ser.serialize_field("baseDelta", v)?;
        }
        if self.uuid != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("uuid", ToString::to_string(&self.uuid).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AoiSyncToMeDelta {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "base_delta",
            "baseDelta",
            "uuid",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BaseDelta,
            Uuid,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "baseDelta" | "base_delta" => Ok(GeneratedField::BaseDelta),
                            "uuid" => Ok(GeneratedField::Uuid),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AoiSyncToMeDelta;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.AoiSyncToMeDelta")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AoiSyncToMeDelta, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut base_delta__ = None;
                let mut uuid__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BaseDelta => {
                            if base_delta__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseDelta"));
                            }
                            base_delta__ = map_.next_value()?;
                        }
                        GeneratedField::Uuid => {
                            if uuid__.is_some() {
                                return Err(serde::de::Error::duplicate_field("uuid"));
                            }
                            uuid__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(AoiSyncToMeDelta {
                    base_delta: base_delta__,
                    uuid: uuid__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.AoiSyncToMeDelta", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Attr {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id != 0 {
            len += 1;
        }
        if !self.raw_data.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.Attr", len)?;
        if self.id != 0 {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if !self.raw_data.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("rawData", pbjson::private::base64::encode(&self.raw_data).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Attr {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "raw_data",
            "rawData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            RawData,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(GeneratedField::Id),
                            "rawData" | "raw_data" => Ok(GeneratedField::RawData),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Attr;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.Attr")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Attr, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut raw_data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::RawData => {
                            if raw_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rawData"));
                            }
                            raw_data__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Attr {
                    id: id__.unwrap_or_default(),
                    raw_data: raw_data__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.Attr", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AttrCollection {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.uuid != 0 {
            len += 1;
        }
        if !self.attrs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.AttrCollection", len)?;
        if self.uuid != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("uuid", ToString::to_string(&self.uuid).as_str())?;
        }
        if !self.attrs.is_empty() {
            struct_ser.serialize_field("attrs", &self.attrs)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AttrCollection {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "uuid",
            "attrs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Uuid,
            Attrs,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "uuid" => Ok(GeneratedField::Uuid),
                            "attrs" => Ok(GeneratedField::Attrs),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AttrCollection;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.AttrCollection")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AttrCollection, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut uuid__ = None;
                let mut attrs__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Uuid => {
                            if uuid__.is_some() {
                                return Err(serde::de::Error::duplicate_field("uuid"));
                            }
                            uuid__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Attrs => {
                            if attrs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("attrs"));
                            }
                            attrs__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(AttrCollection {
                    uuid: uuid__.unwrap_or_default(),
                    attrs: attrs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.AttrCollection", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CharBaseInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.char_id != 0 {
            len += 1;
        }
        if !self.account_id.is_empty() {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        if self.fight_point != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.CharBaseInfo", len)?;
        if self.char_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("charId", ToString::to_string(&self.char_id).as_str())?;
        }
        if !self.account_id.is_empty() {
            struct_ser.serialize_field("accountId", &self.account_id)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if self.fight_point != 0 {
            struct_ser.serialize_field("fightPoint", &self.fight_point)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CharBaseInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "char_id",
            "charId",
            "account_id",
            "accountId",
            "name",
            "fight_point",
            "fightPoint",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CharId,
            AccountId,
            Name,
            FightPoint,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "charId" | "char_id" => Ok(GeneratedField::CharId),
                            "accountId" | "account_id" => Ok(GeneratedField::AccountId),
                            "name" => Ok(GeneratedField::Name),
                            "fightPoint" | "fight_point" => Ok(GeneratedField::FightPoint),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CharBaseInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.CharBaseInfo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CharBaseInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut char_id__ = None;
                let mut account_id__ = None;
                let mut name__ = None;
                let mut fight_point__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CharId => {
                            if char_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("charId"));
                            }
                            char_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AccountId => {
                            if account_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountId"));
                            }
                            account_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FightPoint => {
                            if fight_point__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fightPoint"));
                            }
                            fight_point__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(CharBaseInfo {
                    char_id: char_id__.unwrap_or_default(),
                    account_id: account_id__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                    fight_point: fight_point__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.CharBaseInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CharSerialize {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.char_id != 0 {
            len += 1;
        }
        if self.char_base.is_some() {
            len += 1;
        }
        if self.scene_data.is_some() {
            len += 1;
        }
        if self.item_package.is_some() {
            len += 1;
        }
        if self.r#mod.is_some() {
            len += 1;
        }
        if self.profession_list.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.CharSerialize", len)?;
        if self.char_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("charId", ToString::to_string(&self.char_id).as_str())?;
        }
        if let Some(v) = self.char_base.as_ref() {
            struct_ser.serialize_field("charBase", v)?;
        }
        if let Some(v) = self.scene_data.as_ref() {
            struct_ser.serialize_field("sceneData", v)?;
        }
        if let Some(v) = self.item_package.as_ref() {
            struct_ser.serialize_field("itemPackage", v)?;
        }
        if let Some(v) = self.r#mod.as_ref() {
            struct_ser.serialize_field("mod", v)?;
        }
        if let Some(v) = self.profession_list.as_ref() {
            struct_ser.serialize_field("professionList", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CharSerialize {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "char_id",
            "charId",
            "char_base",
            "charBase",
            "scene_data",
            "sceneData",
            "item_package",
            "itemPackage",
            "mod",
            "profession_list",
            "professionList",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CharId,
            CharBase,
            SceneData,
            ItemPackage,
            Mod,
            ProfessionList,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "charId" | "char_id" => Ok(GeneratedField::CharId),
                            "charBase" | "char_base" => Ok(GeneratedField::CharBase),
                            "sceneData" | "scene_data" => Ok(GeneratedField::SceneData),
                            "itemPackage" | "item_package" => Ok(GeneratedField::ItemPackage),
                            "mod" => Ok(GeneratedField::Mod),
                            "professionList" | "profession_list" => Ok(GeneratedField::ProfessionList),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CharSerialize;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.CharSerialize")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CharSerialize, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut char_id__ = None;
                let mut char_base__ = None;
                let mut scene_data__ = None;
                let mut item_package__ = None;
                let mut r#mod__ = None;
                let mut profession_list__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CharId => {
                            if char_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("charId"));
                            }
                            char_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CharBase => {
                            if char_base__.is_some() {
                                return Err(serde::de::Error::duplicate_field("charBase"));
                            }
                            char_base__ = map_.next_value()?;
                        }
                        GeneratedField::SceneData => {
                            if scene_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sceneData"));
                            }
                            scene_data__ = map_.next_value()?;
                        }
                        GeneratedField::ItemPackage => {
                            if item_package__.is_some() {
                                return Err(serde::de::Error::duplicate_field("itemPackage"));
                            }
                            item_package__ = map_.next_value()?;
                        }
                        GeneratedField::Mod => {
                            if r#mod__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mod"));
                            }
                            r#mod__ = map_.next_value()?;
                        }
                        GeneratedField::ProfessionList => {
                            if profession_list__.is_some() {
                                return Err(serde::de::Error::duplicate_field("professionList"));
                            }
                            profession_list__ = map_.next_value()?;
                        }
                    }
                }
                Ok(CharSerialize {
                    char_id: char_id__.unwrap_or_default(),
                    char_base: char_base__,
                    scene_data: scene_data__,
                    item_package: item_package__,
                    r#mod: r#mod__,
                    profession_list: profession_list__,
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.CharSerialize", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DisappearEntity {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.uuid != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.DisappearEntity", len)?;
        if self.uuid != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("uuid", ToString::to_string(&self.uuid).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DisappearEntity {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "uuid",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Uuid,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "uuid" => Ok(GeneratedField::Uuid),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DisappearEntity;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.DisappearEntity")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DisappearEntity, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut uuid__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Uuid => {
                            if uuid__.is_some() {
                                return Err(serde::de::Error::duplicate_field("uuid"));
                            }
                            uuid__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(DisappearEntity {
                    uuid: uuid__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.DisappearEntity", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EDamageType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Normal => "Normal",
            Self::Miss => "Miss",
            Self::Heal => "Heal",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for EDamageType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "Normal",
            "Miss",
            "Heal",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = EDamageType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "Normal" => Ok(EDamageType::Normal),
                    "Miss" => Ok(EDamageType::Miss),
                    "Heal" => Ok(EDamageType::Heal),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for EEntityType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::EntErrType => "EntErrType",
            Self::EntMonster => "EntMonster",
            Self::EntChar => "EntChar",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for EEntityType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "EntErrType",
            "EntMonster",
            "EntChar",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = EEntityType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "EntErrType" => Ok(EEntityType::EntErrType),
                    "EntMonster" => Ok(EEntityType::EntMonster),
                    "EntChar" => Ok(EEntityType::EntChar),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Entity {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.uuid != 0 {
            len += 1;
        }
        if self.ent_type != 0 {
            len += 1;
        }
        if self.attrs.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.Entity", len)?;
        if self.uuid != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("uuid", ToString::to_string(&self.uuid).as_str())?;
        }
        if self.ent_type != 0 {
            let v = EEntityType::try_from(self.ent_type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.ent_type)))?;
            struct_ser.serialize_field("entType", &v)?;
        }
        if let Some(v) = self.attrs.as_ref() {
            struct_ser.serialize_field("attrs", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Entity {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "uuid",
            "ent_type",
            "entType",
            "attrs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Uuid,
            EntType,
            Attrs,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "uuid" => Ok(GeneratedField::Uuid),
                            "entType" | "ent_type" => Ok(GeneratedField::EntType),
                            "attrs" => Ok(GeneratedField::Attrs),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Entity;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.Entity")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Entity, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut uuid__ = None;
                let mut ent_type__ = None;
                let mut attrs__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Uuid => {
                            if uuid__.is_some() {
                                return Err(serde::de::Error::duplicate_field("uuid"));
                            }
                            uuid__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EntType => {
                            if ent_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entType"));
                            }
                            ent_type__ = Some(map_.next_value::<EEntityType>()? as i32);
                        }
                        GeneratedField::Attrs => {
                            if attrs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("attrs"));
                            }
                            attrs__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Entity {
                    uuid: uuid__.unwrap_or_default(),
                    ent_type: ent_type__.unwrap_or_default(),
                    attrs: attrs__,
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.Entity", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Item {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.mod_new_attr.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.Item", len)?;
        if let Some(v) = self.mod_new_attr.as_ref() {
            struct_ser.serialize_field("modNewAttr", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Item {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "mod_new_attr",
            "modNewAttr",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ModNewAttr,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "modNewAttr" | "mod_new_attr" => Ok(GeneratedField::ModNewAttr),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Item;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.Item")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Item, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut mod_new_attr__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ModNewAttr => {
                            if mod_new_attr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("modNewAttr"));
                            }
                            mod_new_attr__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Item {
                    mod_new_attr: mod_new_attr__,
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.Item", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ItemPackage {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.packages.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.ItemPackage", len)?;
        if !self.packages.is_empty() {
            struct_ser.serialize_field("packages", &self.packages)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ItemPackage {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "packages",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Packages,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "packages" => Ok(GeneratedField::Packages),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ItemPackage;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.ItemPackage")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ItemPackage, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut packages__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Packages => {
                            if packages__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packages"));
                            }
                            packages__ = Some(
                                map_.next_value::<std::collections::HashMap<::pbjson::private::NumberDeserialize<i32>, _>>()?
                                    .into_iter().map(|(k,v)| (k.0, v)).collect()
                            );
                        }
                    }
                }
                Ok(ItemPackage {
                    packages: packages__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.ItemPackage", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Mod {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.mod_infos.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.Mod", len)?;
        if !self.mod_infos.is_empty() {
            struct_ser.serialize_field("modInfos", &self.mod_infos)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Mod {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "mod_infos",
            "modInfos",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ModInfos,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "modInfos" | "mod_infos" => Ok(GeneratedField::ModInfos),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Mod;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.Mod")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Mod, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut mod_infos__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ModInfos => {
                            if mod_infos__.is_some() {
                                return Err(serde::de::Error::duplicate_field("modInfos"));
                            }
                            mod_infos__ = Some(
                                map_.next_value::<std::collections::HashMap<::pbjson::private::NumberDeserialize<i64>, _>>()?
                                    .into_iter().map(|(k,v)| (k.0, v)).collect()
                            );
                        }
                    }
                }
                Ok(Mod {
                    mod_infos: mod_infos__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.Mod", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ModInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.init_link_nums.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.ModInfo", len)?;
        if !self.init_link_nums.is_empty() {
            struct_ser.serialize_field("initLinkNums", &self.init_link_nums)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ModInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "init_link_nums",
            "initLinkNums",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            InitLinkNums,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "initLinkNums" | "init_link_nums" => Ok(GeneratedField::InitLinkNums),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ModInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.ModInfo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ModInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut init_link_nums__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::InitLinkNums => {
                            if init_link_nums__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initLinkNums"));
                            }
                            init_link_nums__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                    }
                }
                Ok(ModInfo {
                    init_link_nums: init_link_nums__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.ModInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ModNewAttr {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.mod_parts.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.ModNewAttr", len)?;
        if !self.mod_parts.is_empty() {
            struct_ser.serialize_field("modParts", &self.mod_parts)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ModNewAttr {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "mod_parts",
            "modParts",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ModParts,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "modParts" | "mod_parts" => Ok(GeneratedField::ModParts),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ModNewAttr;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.ModNewAttr")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ModNewAttr, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut mod_parts__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ModParts => {
                            if mod_parts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("modParts"));
                            }
                            mod_parts__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                    }
                }
                Ok(ModNewAttr {
                    mod_parts: mod_parts__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.ModNewAttr", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Package {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.items.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.Package", len)?;
        if !self.items.is_empty() {
            struct_ser.serialize_field("items", &self.items)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Package {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "items",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Items,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "items" => Ok(GeneratedField::Items),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Package;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.Package")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Package, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut items__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Items => {
                            if items__.is_some() {
                                return Err(serde::de::Error::duplicate_field("items"));
                            }
                            items__ = Some(
                                map_.next_value::<std::collections::HashMap<::pbjson::private::NumberDeserialize<i64>, _>>()?
                                    .into_iter().map(|(k,v)| (k.0, v)).collect()
                            );
                        }
                    }
                }
                Ok(Package {
                    items: items__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.Package", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProfessionList {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.cur_profession_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.ProfessionList", len)?;
        if self.cur_profession_id != 0 {
            struct_ser.serialize_field("curProfessionId", &self.cur_profession_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProfessionList {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "cur_profession_id",
            "curProfessionId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CurProfessionId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "curProfessionId" | "cur_profession_id" => Ok(GeneratedField::CurProfessionId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProfessionList;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.ProfessionList")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProfessionList, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut cur_profession_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CurProfessionId => {
                            if cur_profession_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("curProfessionId"));
                            }
                            cur_profession_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ProfessionList {
                    cur_profession_id: cur_profession_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.ProfessionList", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SceneData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.line_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.SceneData", len)?;
        if self.line_id != 0 {
            struct_ser.serialize_field("lineId", &self.line_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SceneData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "line_id",
            "lineId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            LineId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "lineId" | "line_id" => Ok(GeneratedField::LineId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SceneData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.SceneData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SceneData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut line_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::LineId => {
                            if line_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lineId"));
                            }
                            line_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SceneData {
                    line_id: line_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.SceneData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SkillEffect {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.damages.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.SkillEffect", len)?;
        if !self.damages.is_empty() {
            struct_ser.serialize_field("damages", &self.damages)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SkillEffect {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "damages",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Damages,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "damages" => Ok(GeneratedField::Damages),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SkillEffect;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.SkillEffect")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SkillEffect, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut damages__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Damages => {
                            if damages__.is_some() {
                                return Err(serde::de::Error::duplicate_field("damages"));
                            }
                            damages__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SkillEffect {
                    damages: damages__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.SkillEffect", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SyncContainerData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.v_data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.SyncContainerData", len)?;
        if let Some(v) = self.v_data.as_ref() {
            struct_ser.serialize_field("vData", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SyncContainerData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "v_data",
            "vData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            VData,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "vData" | "v_data" => Ok(GeneratedField::VData),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SyncContainerData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.SyncContainerData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SyncContainerData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut v_data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::VData => {
                            if v_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vData"));
                            }
                            v_data__ = map_.next_value()?;
                        }
                    }
                }
                Ok(SyncContainerData {
                    v_data: v_data__,
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.SyncContainerData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SyncDamageInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.is_miss {
            len += 1;
        }
        if self.r#type != 0 {
            len += 1;
        }
        if self.type_flag != 0 {
            len += 1;
        }
        if self.value != 0 {
            len += 1;
        }
        if self.lucky_value != 0 {
            len += 1;
        }
        if self.hp_lessen_value != 0 {
            len += 1;
        }
        if self.attacker_uuid != 0 {
            len += 1;
        }
        if self.owner_id != 0 {
            len += 1;
        }
        if self.is_dead {
            len += 1;
        }
        if self.top_summoner_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.SyncDamageInfo", len)?;
        if self.is_miss {
            struct_ser.serialize_field("isMiss", &self.is_miss)?;
        }
        if self.r#type != 0 {
            let v = EDamageType::try_from(self.r#type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.r#type)))?;
            struct_ser.serialize_field("type", &v)?;
        }
        if self.type_flag != 0 {
            struct_ser.serialize_field("typeFlag", &self.type_flag)?;
        }
        if self.value != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("value", ToString::to_string(&self.value).as_str())?;
        }
        if self.lucky_value != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("luckyValue", ToString::to_string(&self.lucky_value).as_str())?;
        }
        if self.hp_lessen_value != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("hpLessenValue", ToString::to_string(&self.hp_lessen_value).as_str())?;
        }
        if self.attacker_uuid != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("attackerUuid", ToString::to_string(&self.attacker_uuid).as_str())?;
        }
        if self.owner_id != 0 {
            struct_ser.serialize_field("ownerId", &self.owner_id)?;
        }
        if self.is_dead {
            struct_ser.serialize_field("isDead", &self.is_dead)?;
        }
        if self.top_summoner_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("topSummonerId", ToString::to_string(&self.top_summoner_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SyncDamageInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "is_miss",
            "isMiss",
            "type",
            "type_flag",
            "typeFlag",
            "value",
            "lucky_value",
            "luckyValue",
            "hp_lessen_value",
            "hpLessenValue",
            "attacker_uuid",
            "attackerUuid",
            "owner_id",
            "ownerId",
            "is_dead",
            "isDead",
            "top_summoner_id",
            "topSummonerId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IsMiss,
            Type,
            TypeFlag,
            Value,
            LuckyValue,
            HpLessenValue,
            AttackerUuid,
            OwnerId,
            IsDead,
            TopSummonerId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "isMiss" | "is_miss" => Ok(GeneratedField::IsMiss),
                            "type" => Ok(GeneratedField::Type),
                            "typeFlag" | "type_flag" => Ok(GeneratedField::TypeFlag),
                            "value" => Ok(GeneratedField::Value),
                            "luckyValue" | "lucky_value" => Ok(GeneratedField::LuckyValue),
                            "hpLessenValue" | "hp_lessen_value" => Ok(GeneratedField::HpLessenValue),
                            "attackerUuid" | "attacker_uuid" => Ok(GeneratedField::AttackerUuid),
                            "ownerId" | "owner_id" => Ok(GeneratedField::OwnerId),
                            "isDead" | "is_dead" => Ok(GeneratedField::IsDead),
                            "topSummonerId" | "top_summoner_id" => Ok(GeneratedField::TopSummonerId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SyncDamageInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.SyncDamageInfo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SyncDamageInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut is_miss__ = None;
                let mut r#type__ = None;
                let mut type_flag__ = None;
                let mut value__ = None;
                let mut lucky_value__ = None;
                let mut hp_lessen_value__ = None;
                let mut attacker_uuid__ = None;
                let mut owner_id__ = None;
                let mut is_dead__ = None;
                let mut top_summoner_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::IsMiss => {
                            if is_miss__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isMiss"));
                            }
                            is_miss__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map_.next_value::<EDamageType>()? as i32);
                        }
                        GeneratedField::TypeFlag => {
                            if type_flag__.is_some() {
                                return Err(serde::de::Error::duplicate_field("typeFlag"));
                            }
                            type_flag__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::LuckyValue => {
                            if lucky_value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("luckyValue"));
                            }
                            lucky_value__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::HpLessenValue => {
                            if hp_lessen_value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hpLessenValue"));
                            }
                            hp_lessen_value__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AttackerUuid => {
                            if attacker_uuid__.is_some() {
                                return Err(serde::de::Error::duplicate_field("attackerUuid"));
                            }
                            attacker_uuid__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::OwnerId => {
                            if owner_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ownerId"));
                            }
                            owner_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::IsDead => {
                            if is_dead__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isDead"));
                            }
                            is_dead__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TopSummonerId => {
                            if top_summoner_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("topSummonerId"));
                            }
                            top_summoner_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SyncDamageInfo {
                    is_miss: is_miss__.unwrap_or_default(),
                    r#type: r#type__.unwrap_or_default(),
                    type_flag: type_flag__.unwrap_or_default(),
                    value: value__.unwrap_or_default(),
                    lucky_value: lucky_value__.unwrap_or_default(),
                    hp_lessen_value: hp_lessen_value__.unwrap_or_default(),
                    attacker_uuid: attacker_uuid__.unwrap_or_default(),
                    owner_id: owner_id__.unwrap_or_default(),
                    is_dead: is_dead__.unwrap_or_default(),
                    top_summoner_id: top_summoner_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.SyncDamageInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SyncNearDeltaInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delta_infos.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.SyncNearDeltaInfo", len)?;
        if !self.delta_infos.is_empty() {
            struct_ser.serialize_field("deltaInfos", &self.delta_infos)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SyncNearDeltaInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delta_infos",
            "deltaInfos",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DeltaInfos,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "deltaInfos" | "delta_infos" => Ok(GeneratedField::DeltaInfos),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SyncNearDeltaInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.SyncNearDeltaInfo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SyncNearDeltaInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delta_infos__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DeltaInfos => {
                            if delta_infos__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deltaInfos"));
                            }
                            delta_infos__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SyncNearDeltaInfo {
                    delta_infos: delta_infos__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.SyncNearDeltaInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SyncNearEntities {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.appear.is_empty() {
            len += 1;
        }
        if !self.disappear.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.SyncNearEntities", len)?;
        if !self.appear.is_empty() {
            struct_ser.serialize_field("appear", &self.appear)?;
        }
        if !self.disappear.is_empty() {
            struct_ser.serialize_field("disappear", &self.disappear)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SyncNearEntities {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "appear",
            "disappear",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Appear,
            Disappear,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "appear" => Ok(GeneratedField::Appear),
                            "disappear" => Ok(GeneratedField::Disappear),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SyncNearEntities;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.SyncNearEntities")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SyncNearEntities, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut appear__ = None;
                let mut disappear__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Appear => {
                            if appear__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appear"));
                            }
                            appear__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Disappear => {
                            if disappear__.is_some() {
                                return Err(serde::de::Error::duplicate_field("disappear"));
                            }
                            disappear__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SyncNearEntities {
                    appear: appear__.unwrap_or_default(),
                    disappear: disappear__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.SyncNearEntities", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SyncToMeDeltaInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.delta_info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.SyncToMeDeltaInfo", len)?;
        if let Some(v) = self.delta_info.as_ref() {
            struct_ser.serialize_field("deltaInfo", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SyncToMeDeltaInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delta_info",
            "deltaInfo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DeltaInfo,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "deltaInfo" | "delta_info" => Ok(GeneratedField::DeltaInfo),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SyncToMeDeltaInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.SyncToMeDeltaInfo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SyncToMeDeltaInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delta_info__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DeltaInfo => {
                            if delta_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deltaInfo"));
                            }
                            delta_info__ = map_.next_value()?;
                        }
                    }
                }
                Ok(SyncToMeDeltaInfo {
                    delta_info: delta_info__,
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.SyncToMeDeltaInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Vector3 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.x != 0. {
            len += 1;
        }
        if self.y != 0. {
            len += 1;
        }
        if self.z != 0. {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("blueprotobuf_package.Vector3", len)?;
        if self.x != 0. {
            struct_ser.serialize_field("x", &self.x)?;
        }
        if self.y != 0. {
            struct_ser.serialize_field("y", &self.y)?;
        }
        if self.z != 0. {
            struct_ser.serialize_field("z", &self.z)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Vector3 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "x",
            "y",
            "z",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            X,
            Y,
            Z,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "x" => Ok(GeneratedField::X),
                            "y" => Ok(GeneratedField::Y),
                            "z" => Ok(GeneratedField::Z),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Vector3;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct blueprotobuf_package.Vector3")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Vector3, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut x__ = None;
                let mut y__ = None;
                let mut z__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::X => {
                            if x__.is_some() {
                                return Err(serde::de::Error::duplicate_field("x"));
                            }
                            x__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Y => {
                            if y__.is_some() {
                                return Err(serde::de::Error::duplicate_field("y"));
                            }
                            y__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Z => {
                            if z__.is_some() {
                                return Err(serde::de::Error::duplicate_field("z"));
                            }
                            z__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Vector3 {
                    x: x__.unwrap_or_default(),
                    y: y__.unwrap_or_default(),
                    z: z__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("blueprotobuf_package.Vector3", FIELDS, GeneratedVisitor)
    }
}

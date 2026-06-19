UPDATE emulators
SET
    extra_files = (
        SELECT
            jsonb_agg (
                jsonb_build_object (
                    's3_path',
                    elem ->> 's3_path',
                    'path',
                    COALESCE(
                        elem ->> 'path',
                        CASE platform
                            WHEN 'windows' THEN elem ->> 'windows_path'
                            WHEN 'linux' THEN elem ->> 'linux_path'
                            WHEN 'macos' THEN elem ->> 'macos_path'
                            ELSE elem ->> 'windows_path'
                        END,
                        ''
                    )
                )
            )
        FROM
            jsonb_array_elements (extra_files) AS elem
    )
WHERE
    jsonb_typeof (extra_files) = 'array'
    AND jsonb_array_length (extra_files) > 0;
